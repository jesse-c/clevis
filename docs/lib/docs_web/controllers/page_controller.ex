defmodule DocsWeb.PageController do
  use DocsWeb, :controller

  def home(conn, _params) do
    # The home page is often custom made,
    # so skip the default app layout.
    render(conn, :home, layout: false)
  end

  def example(conn, _params) do
    render(conn, :example, layout: {DocsWeb.Layouts, :page})
  end

  def example2(conn, _params) do
    render(conn, :example2, layout: {DocsWeb.Layouts, :page})
  end
end
