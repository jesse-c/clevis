defmodule Docs.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      DocsWeb.Telemetry,
      {DNSCluster, query: Application.get_env(:docs, :dns_cluster_query) || :ignore},
      {Phoenix.PubSub, name: Docs.PubSub},
      # Start a worker by calling: Docs.Worker.start_link(arg)
      # {Docs.Worker, arg},
      # Start to serve requests, typically the last entry
      DocsWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Docs.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    DocsWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
