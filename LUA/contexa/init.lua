--- contexa — package entry point.
--- @module contexa

local models = require("contexa.models")
local workspace = require("contexa.workspace")

return {
    -- Workspace
    GCCWorkspace = workspace,

    -- Models
    new_ota = models.new_ota,
    new_commit = models.new_commit,
    new_branch_metadata = models.new_branch_metadata,
    new_context_result = models.new_context_result,

    -- Serialization
    ota_to_markdown = models.ota_to_markdown,
    commit_to_markdown = models.commit_to_markdown,
    metadata_to_yaml = models.metadata_to_yaml,
    metadata_from_yaml = models.metadata_from_yaml,
    context_summary = models.context_summary,

    -- Utilities
    generate_id = models.generate_id,
    timestamp = models.timestamp,
}
