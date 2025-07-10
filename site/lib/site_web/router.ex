defmodule SiteWeb.Router do
  use SiteWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {SiteWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", SiteWeb do
    pipe_through :browser

    get "/", PageController, :home
    get "/features", PageController, :features
    get "/readers", PageController, :readers
  end

  scope "/", SiteWeb do
    pipe_through :api

    get "/healthz", HealthController, :healthz
  end
end
