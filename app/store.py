from __future__ import annotations

import json
from typing import Protocol

from redis import Redis

from app.models import FactRecord, ObservationRecord, PlaybookRecord


class ContextStore(Protocol):
    def put_fact(self, record: FactRecord) -> FactRecord:
        ...

    def put_playbook(self, record: PlaybookRecord) -> PlaybookRecord:
        ...

    def put_observation(self, record: ObservationRecord) -> ObservationRecord:
        ...

    def list_facts(self, namespace: str, tool_name: str, resource_id: str | None = None) -> list[FactRecord]:
        ...

    def list_playbooks(
        self,
        namespace: str,
        tool_name: str,
        error_family: str | None = None,
    ) -> list[PlaybookRecord]:
        ...

    def list_observations(
        self,
        namespace: str,
        tool_name: str,
        resource_id: str | None = None,
    ) -> list[ObservationRecord]:
        ...


class RedisContextStore:
    def __init__(self, client: Redis, ttl_seconds: int) -> None:
        self.client = client
        self.ttl_seconds = ttl_seconds

    @staticmethod
    def _segment(value: str | None) -> str:
        return value if value else "*"

    def _fact_key(self, record: FactRecord) -> str:
        return ":".join(
            ["iterum", "fact", record.namespace, record.tool_name, self._segment(record.resource_id), record.id]
        )

    def _playbook_key(self, record: PlaybookRecord) -> str:
        return ":".join(["iterum", "playbook", record.namespace, record.tool_name, record.id])

    def _observation_key(self, record: ObservationRecord) -> str:
        return ":".join(
            ["iterum", "observation", record.namespace, record.tool_name, self._segment(record.resource_id), record.id]
        )

    def _fact_tool_index(self, namespace: str, tool_name: str) -> str:
        return ":".join(["iterum", "index", "facts", namespace, tool_name])

    def _fact_resource_index(self, namespace: str, tool_name: str, resource_id: str | None) -> str:
        return ":".join(["iterum", "index", "facts", namespace, tool_name, self._segment(resource_id)])

    def _playbook_tool_index(self, namespace: str, tool_name: str) -> str:
        return ":".join(["iterum", "index", "playbooks", namespace, tool_name])

    def _playbook_family_index(self, namespace: str, tool_name: str, error_family: str | None) -> str:
        return ":".join(["iterum", "index", "playbooks", namespace, tool_name, self._segment(error_family)])

    def _observation_tool_index(self, namespace: str, tool_name: str) -> str:
        return ":".join(["iterum", "index", "observations", namespace, tool_name])

    def _observation_resource_index(self, namespace: str, tool_name: str, resource_id: str | None) -> str:
        return ":".join(["iterum", "index", "observations", namespace, tool_name, self._segment(resource_id)])

    def _get_records(self, keys: list[str], model: type[FactRecord] | type[PlaybookRecord] | type[ObservationRecord]) -> list:
        entries = []
        for key in keys:
            payload = self.client.get(key)
            if payload:
                entries.append(model.model_validate(json.loads(payload)))
        return entries

    def put_fact(self, record: FactRecord) -> FactRecord:
        key = self._fact_key(record)
        self.client.set(key, record.model_dump_json())
        self.client.sadd(self._fact_tool_index(record.namespace, record.tool_name), key)
        self.client.sadd(self._fact_resource_index(record.namespace, record.tool_name, record.resource_id), key)
        return record

    def put_playbook(self, record: PlaybookRecord) -> PlaybookRecord:
        key = self._playbook_key(record)
        self.client.set(key, record.model_dump_json())
        self.client.sadd(self._playbook_tool_index(record.namespace, record.tool_name), key)
        self.client.sadd(self._playbook_family_index(record.namespace, record.tool_name, record.error_family), key)
        return record

    def put_observation(self, record: ObservationRecord) -> ObservationRecord:
        key = self._observation_key(record)
        ttl = record.ttl_seconds if record.ttl_seconds is not None else self.ttl_seconds
        self.client.set(key, record.model_dump_json(), ex=ttl)
        self.client.sadd(self._observation_tool_index(record.namespace, record.tool_name), key)
        self.client.sadd(self._observation_resource_index(record.namespace, record.tool_name, record.resource_id), key)
        return record

    def list_facts(self, namespace: str, tool_name: str, resource_id: str | None = None) -> list[FactRecord]:
        keys = set(self.client.smembers(self._fact_tool_index(namespace, tool_name)))
        if resource_id is not None:
            keys.update(self.client.smembers(self._fact_resource_index(namespace, tool_name, resource_id)))
            keys.update(self.client.smembers(self._fact_resource_index(namespace, tool_name, None)))
        return self._get_records(sorted(keys), FactRecord)

    def list_playbooks(self, namespace: str, tool_name: str, error_family: str | None = None) -> list[PlaybookRecord]:
        keys = set(self.client.smembers(self._playbook_tool_index(namespace, tool_name)))
        if error_family is not None:
            keys.update(self.client.smembers(self._playbook_family_index(namespace, tool_name, error_family)))
            keys.update(self.client.smembers(self._playbook_family_index(namespace, tool_name, None)))
        return self._get_records(sorted(keys), PlaybookRecord)

    def list_observations(self, namespace: str, tool_name: str, resource_id: str | None = None) -> list[ObservationRecord]:
        keys = set(self.client.smembers(self._observation_tool_index(namespace, tool_name)))
        if resource_id is not None:
            keys.update(self.client.smembers(self._observation_resource_index(namespace, tool_name, resource_id)))
            keys.update(self.client.smembers(self._observation_resource_index(namespace, tool_name, None)))
        return self._get_records(sorted(keys), ObservationRecord)


class InMemoryContextStore:
    def __init__(self) -> None:
        self.facts: dict[tuple[str, str, str], FactRecord] = {}
        self.playbooks: dict[tuple[str, str, str], PlaybookRecord] = {}
        self.observations: dict[tuple[str, str, str], ObservationRecord] = {}

    def put_fact(self, record: FactRecord) -> FactRecord:
        self.facts[(record.namespace, record.tool_name, record.id)] = record
        return record

    def put_playbook(self, record: PlaybookRecord) -> PlaybookRecord:
        self.playbooks[(record.namespace, record.tool_name, record.id)] = record
        return record

    def put_observation(self, record: ObservationRecord) -> ObservationRecord:
        self.observations[(record.namespace, record.tool_name, record.id)] = record
        return record

    def list_facts(self, namespace: str, tool_name: str, resource_id: str | None = None) -> list[FactRecord]:
        return [
            record
            for record in self.facts.values()
            if record.namespace == namespace
            and record.tool_name == tool_name
            and (resource_id is None or record.resource_id in {resource_id, None})
        ]

    def list_playbooks(self, namespace: str, tool_name: str, error_family: str | None = None) -> list[PlaybookRecord]:
        return [
            record
            for record in self.playbooks.values()
            if record.namespace == namespace
            and record.tool_name == tool_name
            and (error_family is None or record.error_family in {error_family, None})
        ]

    def list_observations(self, namespace: str, tool_name: str, resource_id: str | None = None) -> list[ObservationRecord]:
        return [
            record
            for record in self.observations.values()
            if record.namespace == namespace
            and record.tool_name == tool_name
            and (resource_id is None or record.resource_id in {resource_id, None})
        ]
