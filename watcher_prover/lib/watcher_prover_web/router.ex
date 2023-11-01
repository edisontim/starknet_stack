defmodule WatcherProverWeb.Router do
  use WatcherProverWeb, :router

  pipeline :browser do
    plug(:accepts, ["html"])
    plug(:fetch_session)
    plug(:fetch_live_flash)
    plug(:put_root_layout, html: {WatcherProverWeb.Layouts, :root})
    plug(:protect_from_forgery)
    plug(:put_secure_browser_headers)
  end

  pipeline :api do
    plug(:accepts, ["json"])
  end

  scope "/", WatcherProverWeb do
    pipe_through(:browser)

    get("/", PageController, :home)
  end

  scope "/get_current_inscription_id", WatcherProverWeb do
    get("/", PageController, :get_current_inscription_id)
  end

  scope "/get_inscription_id_by_block_hash/:block_hash", WatcherProverWeb do
    get("/", PageController, :get_inscription_id_by_block_hash)
  end

  # Other scopes may use custom stacks.
  # scope "/api", WatcherProverWeb do
  #   pipe_through :api
  # end

  # Enable LiveDashboard and Swoosh mailbox preview in development
  if Application.compile_env(:watcher_prover, :dev_routes) do
    # If you want to use the LiveDashboard in production, you should put
    # it behind authentication and allow only admins to access it.
    # If your application does not have an admins-only section yet,
    # you can use Plug.BasicAuth to set up some basic authentication
    # as long as you are also using SSL (which you should anyway).
    import Phoenix.LiveDashboard.Router

    scope "/dev" do
      pipe_through(:browser)

      live_dashboard("/dashboard", metrics: WatcherProverWeb.Telemetry)
      forward("/mailbox", Plug.Swoosh.MailboxPreview)
    end
  end
end
