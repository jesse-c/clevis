defmodule DocsWeb.PageController do
  use DocsWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
