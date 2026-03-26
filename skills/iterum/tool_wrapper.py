"""
OpenClaw AI Tool Wrapper for Iterum
Automatically injects context before brittle tool calls
"""

import os
import subprocess
import json
from typing import Optional, Dict, Any


class IterumToolWrapper:
    """Wraps brittle tools to automatically consult Iterum"""

    BRITTLE_TOOLS = ["sqlite3", "psql", "mysql", "curl", "wget", "http"]
    SCRIPT_DIR = os.path.join(os.path.dirname(__file__), "scripts")

    def __init__(self, base_url: str = "http://127.0.0.1:8000"):
        self.base_url = base_url
        os.environ["ITERUM_BASE_URL"] = base_url

    def should_wrap(self, tool_name: str) -> bool:
        """Check if this tool should consult Iterum"""
        return any(brittle in tool_name.lower() for brittle in self.BRITTLE_TOOLS)

    def retrieve_context(
        self,
        tool_name: str,
        resource_id: str,
        error_text: Optional[str] = None,
        namespace: str = "default"
    ) -> Dict[str, Any]:
        """Retrieve context from Iterum before tool execution"""
        script = os.path.join(self.SCRIPT_DIR, "retrieve_context.sh")

        cmd = [script, namespace, tool_name, resource_id]
        if error_text:
            cmd.append(error_text)

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=5
            )

            if result.returncode == 0:
                return json.loads(result.stdout)
            else:
                # Iterum unavailable, continue without context
                return {"prompt_context": "Iterum unavailable"}

        except Exception:
            return {"prompt_context": "Iterum unavailable"}

    def store_fact(
        self,
        tool_name: str,
        resource_id: str,
        topic: str,
        title: str,
        content: str,
        confidence: float = 1.0,
        namespace: str = "default"
    ) -> bool:
        """Store a learned fact to Iterum"""
        script = os.path.join(self.SCRIPT_DIR, "store_fact.sh")

        try:
            subprocess.run(
                [script, namespace, tool_name, resource_id, topic, title, content, str(confidence)],
                capture_output=True,
                timeout=5,
                check=False
            )
            return True
        except Exception:
            return False

    def store_observation(
        self,
        tool_name: str,
        resource_id: str,
        topic: str,
        content: str,
        confidence: float = 0.9,
        namespace: str = "default"
    ) -> bool:
        """Store an observation to Iterum"""
        script = os.path.join(self.SCRIPT_DIR, "store_observation.sh")

        try:
            subprocess.run(
                [script, namespace, tool_name, resource_id, topic, content, str(confidence)],
                capture_output=True,
                timeout=5,
                check=False
            )
            return True
        except Exception:
            return False

    def wrap_tool_call(
        self,
        tool_name: str,
        resource_id: str,
        original_prompt: str
    ) -> str:
        """Wrap a tool call with Iterum context"""

        if not self.should_wrap(tool_name):
            return original_prompt

        # Retrieve context
        context = self.retrieve_context(tool_name, resource_id)
        prompt_context = context.get("prompt_context", "")

        if prompt_context and prompt_context != "No relevant stored context.":
            # Inject context into prompt
            enhanced_prompt = f"""{original_prompt}

IMPORTANT - Known context from previous runs:
{prompt_context}

Use this context to avoid known errors and use correct schemas/patterns.
"""
            return enhanced_prompt

        return original_prompt


# Singleton instance
_wrapper = None

def get_wrapper() -> IterumToolWrapper:
    """Get or create the Iterum wrapper singleton"""
    global _wrapper
    if _wrapper is None:
        _wrapper = IterumToolWrapper()
    return _wrapper
