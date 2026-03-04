defmodule Contexa do
  @moduledoc """
  contexa — Git-inspired context management for LLM agents.

  COMMIT, BRANCH, MERGE, and CONTEXT operations over a persistent
  versioned memory workspace. Based on arXiv:2508.00031.

  ## Quick Start

      ws = Contexa.Workspace.new("/tmp/my-project")
      ws = Contexa.Workspace.init(ws, "Build an AI agent")

      {ws, _ota} = Contexa.Workspace.log_ota(ws, "Observed X", "Thinking Y", "Did Z")
      {ws, _commit} = Contexa.Workspace.commit(ws, "Implemented feature")

      ws = Contexa.Workspace.branch(ws, "explore", "Try alternative approach")
      {ws, _commit} = Contexa.Workspace.merge(ws, "explore")

      ctx = Contexa.Workspace.context(ws, "main", 3)
      IO.puts(Contexa.Models.context_summary(ctx))
  """
end
