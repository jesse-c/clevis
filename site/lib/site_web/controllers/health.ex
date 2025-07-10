defmodule SiteWeb.HealthController do
  use SiteWeb, :controller

  def healthz(conn, _params) do
    conn
    |> put_status(200)
    |> json(%{status: "ok"})
  end
end
