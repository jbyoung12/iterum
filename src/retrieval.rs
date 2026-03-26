use crate::error::AppError;
use crate::formatter::format_prompt_context;
use crate::models::*;
use crate::ranking::*;
use crate::store::Store;

pub fn infer_error_family(error_text: Option<&str>) -> Option<String> {
    let text = error_text?.to_lowercase();
    if text.contains("no such column") || text.contains("unknown column") {
        return Some("unknown_column".into());
    }
    if text.contains("no such table") {
        return Some("unknown_table".into());
    }
    if text.contains("not found") || text.contains("404") {
        return Some("not_found".into());
    }
    if text.contains("invalid url") || text.contains("not a url") {
        return Some("invalid_url".into());
    }
    if text.contains("missing required") {
        return Some("missing_required_field".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_error_family() {
        assert_eq!(
            infer_error_family(Some("Error: no such column: foo")),
            Some("unknown_column".into())
        );
        assert_eq!(
            infer_error_family(Some("no such table: bar")),
            Some("unknown_table".into())
        );
        assert_eq!(
            infer_error_family(Some("HTTP 404 Not Found")),
            Some("not_found".into())
        );
        assert_eq!(
            infer_error_family(Some("invalid url provided")),
            Some("invalid_url".into())
        );
        assert_eq!(
            infer_error_family(Some("missing required field: name")),
            Some("missing_required_field".into())
        );
        assert_eq!(infer_error_family(Some("everything is fine")), None);
        assert_eq!(infer_error_family(None), None);
    }
}

pub struct ContextRetriever<S: Store> {
    store: S,
    max_items: usize,
}

impl<S: Store> ContextRetriever<S> {
    pub fn new(store: S, max_items: usize) -> Self {
        Self { store, max_items }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub async fn retrieve(
        &self,
        request: &ContextRetrieveRequest,
    ) -> Result<ContextRetrieveResponse, AppError> {
        let error_family = infer_error_family(request.error_text.as_deref());

        let facts = self
            .store
            .list_facts(
                &request.namespace,
                &request.tool_name,
                request.resource_id.as_deref(),
            )
            .await?;
        let playbooks = self
            .store
            .list_playbooks(
                &request.namespace,
                &request.tool_name,
                error_family.as_deref(),
            )
            .await?;
        let observations = self
            .store
            .list_observations(
                &request.namespace,
                &request.tool_name,
                request.resource_id.as_deref(),
            )
            .await?;
        let constraints = self
            .store
            .list_constraints(
                &request.namespace,
                &request.tool_name,
                request.resource_id.as_deref(),
            )
            .await?;
        let failure_patterns = self
            .store
            .list_failure_patterns(
                &request.namespace,
                &request.tool_name,
                request.resource_id.as_deref(),
            )
            .await?;

        let ranked_facts = rank_facts(facts, request, self.max_items);
        let ranked_playbooks = rank_playbooks(playbooks, error_family.as_deref(), self.max_items);
        let ranked_observations = rank_observations(observations, request, self.max_items);
        let ranked_constraints = rank_constraints(constraints, request, self.max_items);
        let ranked_failure_patterns =
            rank_failure_patterns(failure_patterns, request, self.max_items);

        let prompt_context = format_prompt_context(
            &ranked_facts,
            &ranked_playbooks,
            &ranked_observations,
            &ranked_constraints,
            &ranked_failure_patterns,
        );

        Ok(ContextRetrieveResponse {
            namespace: request.namespace.clone(),
            tool_name: request.tool_name.clone(),
            resource_id: request.resource_id.clone(),
            matched_error_family: error_family,
            facts: ranked_facts,
            playbooks: ranked_playbooks,
            observations: ranked_observations,
            constraints: ranked_constraints,
            failure_patterns: ranked_failure_patterns,
            prompt_context,
        })
    }
}
