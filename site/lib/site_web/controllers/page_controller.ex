defmodule SiteWeb.PageController do
  use SiteWeb, :controller

  def home(conn, _params) do
    render(conn, :home, page_title: "Home")
  end

  def features(conn, _params) do
    render(conn, :features, page_title: "Features")
  end

  def readers(conn, _params) do
    render(conn, :readers, page_title: "Readers")
  end
end
