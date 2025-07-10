defmodule SiteWeb.PageController do
  use SiteWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end

  def features(conn, _params) do
    render(conn, :features)
  end

  def readers(conn, _params) do
    render(conn, :readers)
  end
end
