defmodule Site.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      SiteWeb.Telemetry,
      {DNSCluster, query: Application.get_env(:site, :dns_cluster_query) || :ignore},
      {Phoenix.PubSub, name: Site.PubSub},
      # Start a worker by calling: Site.Worker.start_link(arg)
      # {Site.Worker, arg},
      # Start to serve requests, typically the last entry
      SiteWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Site.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    SiteWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
