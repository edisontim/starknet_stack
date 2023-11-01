defmodule WatcherProverWeb.PageController do
  use WatcherProverWeb, :controller
  require Logger

  def home(_conn, _params) do
  end

  def get_current_inscription_id(conn, _params) do
    json(conn, %{inscription_id: elem(File.read("./blocks/current_inscription_id"), 1)})
  end

  def get_inscription_id_by_block_hash(conn, _params) do
    block_hash = conn.params["block_hash"]
    test = CubDB.get(CubDB, "tests")
    # change the value returned in the json
    json(conn, %{block_hash: "ff06f3370b5393c990533baefd80bef08d47cdaff8088246c1359db1366d60fei0"})
  end
end
