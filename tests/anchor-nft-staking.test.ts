import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getAccount } from "@solana/spl-token";
import { expect } from "chai";
import { AnchorNftStaking } from "../target/types/anchor_nft_staking";
import { setupNft } from "./utils";

describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet;
  const METADATA_PROGRAM_ID = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
  const program = anchor.workspace
    .AnchorNftStaking as Program<AnchorNftStaking>;

  let delegatedAuthPda: anchor.web3.PublicKey;
  let stakeStatePda: anchor.web3.PublicKey;
  let nft: any;
  let mintAuth: anchor.web3.PublicKey;
  let mint: anchor.web3.PublicKey;
  let tokenAddress: anchor.web3.PublicKey;

  before(async () => {
    ({ nft, delegatedAuthPda, stakeStatePda, mint, mintAuth, tokenAddress } =
      await setupNft(program, wallet.payer));
    nft = nft;
    delegatedAuthPda = delegatedAuthPda;
    stakeStatePda = stakeStatePda;
    mint = mint;
    mintAuth = mintAuth;
    tokenAddress = tokenAddress;
  });

  it("Stakes", async () => {
    // Add your test here.
    await program.methods
      .stake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .rpc();

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Staked");
  });

  it("Redeems", async () => {
    await program.methods
      .redeem()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc();
    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Unstaked");
    const tokenAccount = await getAccount(provider.connection, tokenAddress);
  });
});
