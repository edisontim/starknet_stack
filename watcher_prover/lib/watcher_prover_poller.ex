defmodule WatcherProver.Poller do
  use GenServer
  use Tesla
  alias WatcherProver.NIF
  plug(Tesla.Middleware.BaseUrl, "http://localhost:5000")
  plug(Tesla.Middleware.JSON)
  alias WatcherProver.Rpc

  require Logger
  alias WatcherProver.S3

  @polling_frequency_ms 15_000
  @number_of_blocks_for_confirmation 0

  def start_link(args) do
    GenServer.start_link(__MODULE__, args)
  end

  @impl true
  def init(_opts) do
    state = %{
      last_confirmed_block_number: 1,
      prev_inscription_id: "0000000000000000000000000000000000000000000000000000000000000000i0",
      highest_block: 0
    }

    Process.send_after(self(), :poll, 10)

    :ok = File.mkdir_p("./blocks")

    {:ok, state}
  end

  @doc """
  This handler will first poll the chain for the latest block number, check which blocks are confirmed but have not
  been proved yet, then run a proof for them and upload it to S3.
  """
  @impl true
  def handle_info(
        :poll,
        state = %{
          last_confirmed_block_number: last_confirmed_block_number,
          prev_inscription_id: prev_inscription_id
        }
      ) do
    # {:ok, current_block_height} = Rpc.last_block_number()

    # Logger.info("Previous inscription id #{prev_inscription_id}")

    Process.send_after(self(), :poll, @polling_frequency_ms)

    # posting_res = inscribe_file("./blocks/test-block.txt", 10)

    # If the process exited correctly
    # prev_inscription_id =
    #   if elem(posting_res, 1) == 0 do
    #     Poison.decode!(elem(posting_res, 0))["inscription"]
    #   else
    #     prev_inscription_id
    #   end

    # Logger.info("Return of program: #{prev_inscription_id}")
    # File.write("./blocks/current_inscription_id", prev_inscription_id, [:write])
    CubDB.put(CubDB, "tests", "lol")

    {:noreply,
     %{
       state
       | last_confirmed_block_number: 1,
         prev_inscription_id: prev_inscription_id
     }}

    # if last_confirmed_block_number + @number_of_blocks_for_confirmation <= current_block_height do
    #   {:ok, block} = Rpc.get_block_by_number(last_confirmed_block_number)

    # Logger.info("Running proof for block #{block["block_hash"]} with contents #{inspect(block)}")

    # TODO: fetch executions from the invoke transactions for this block to prove
    # {:ok, program} = File.read("./programs/cairo0.json")

    # {proof, public_inputs} = run_proofs(program)

    # Logger.info("Generated block proof #{inspect(proof)}")

    # block_hash = block["block_hash"]

    # Can't concat variables as this changes the final bytes written to file, not sure why
    # file_path = "./blocks/#{block_hash}-block.txt"

    # :ok = File.write(file_path, prev_inscription_id, [:write])
    # :ok = File.write(file_path, public_inputs, [:append])
    # :ok = File.write(file_path, "public_inputs_end", [:append])
    # :ok = File.write(file_path, proof, [:append])

    #   Logger.info("Saved block with id #{block_hash} to file #{file_path}")

    #   {:noreply,
    #    %{
    #      state
    #      | last_confirmed_block_number: last_confirmed_block_number + 1
    #    }}
    # else
    #   {:noreply,
    #    %{
    #      state
    #      | last_confirmed_block_number: last_confirmed_block_number
    #    }}
    # end
  end

  def run_proofs(block) do
    NIF.run_program_and_get_proof(block)
  end

  def inscribe_file(block_path, fee_rate) do
    System.cmd("ord", [
      "--regtest",
      "--rpc-url",
      "localhost:18332",
      "--bitcoin-rpc-user",
      "tim",
      "--bitcoin-rpc-pass",
      "tim",
      "wallet",
      "inscribe",
      "--fee-rate",
      Integer.to_string(fee_rate),
      block_path
    ])
  end
end
