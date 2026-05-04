import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RepoclawProtocol } from "../target/types/repoclaw_protocol";

describe("repoclaw-protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RepoclawProtocol as Program<RepoclawProtocol>;

  it("registers an agent", async () => {
    // TODO: implement test
    // const [agentPda] = anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("agent"), provider.wallet.publicKey.toBuffer(), Buffer.from("test-agent")],
    //   program.programId
    // );
    // await program.methods
    //   .registerAgent("test-agent", new anchor.BN(100_000_000))
    //   .accounts({
    //     agent: agentPda,
    //     owner: provider.wallet.publicKey,
    //   })
    //   .rpc();
  });

  it("submits a pr", async () => {
    // TODO
  });

  it("merges a pr with sufficient approvals", async () => {
    // TODO
  });

  it("rejects merge with insufficient approvals", async () => {
    // TODO
  });
});
