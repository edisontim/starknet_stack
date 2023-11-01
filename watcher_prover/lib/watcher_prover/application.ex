defmodule WatcherProver.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @db_data_dir "./blocks/db"
  @impl true
  def start(_type, _args) do
    children = [
      # Start the Telemetry supervisor
      WatcherProverWeb.Telemetry,
      # Start the Ecto repository
      # WatcherProver.Repo,
      # Start the PubSub system
      {Phoenix.PubSub, name: WatcherProver.PubSub},
      # Start Finch
      {Finch, name: WatcherProver.Finch},
      # Start the Endpoint (http/https)
      WatcherProverWeb.Endpoint,
      # Start a worker by calling: WatcherProver.Worker.start_link(arg)
      # {WatcherProver.Worker, arg}
      WatcherProver.Poller,
      {CubDB, [data_dir: @db_data_dir, name: CubDB]}
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: WatcherProver.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    WatcherProverWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
