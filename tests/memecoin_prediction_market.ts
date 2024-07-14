import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { MemecoinPredictionMarket } from "../target/types/memecoin_prediction_market";
import { TOKEN_PROGRAM_ID, createMint, mintTo, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { assert } from "chai"

describe("memecoin_prediction_market", () => {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MemecoinPredictionMarket as Program<MemecoinPredictionMarket>;

  let marketPda: anchor.web3.PublicKey;
  let marketBump: number;
  let marketTokenAccount: anchor.web3.PublicKey;
  let betPda: anchor.web3.PublicKey;
  let betBump: number;
  let mint: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;

  const marketName = "DOGE_USD";
  const creator = anchor.web3.Keypair.generate();
  const user = anchor.web3.Keypair.generate();
  const betAmount = new anchor.BN(1_000_000); // 1 token
  const prediction = true; // Price will go up

  before(async () => {
    // Airdrop SOL to creator and user
    await provider.connection.requestAirdrop(creator.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    await new Promise(resolve => setTimeout(resolve, 500));

    // Create mint
    mint = await createMint(
      provider.connection,
      creator,
      creator.publicKey,
      null,
      6
    );

    // Create user token account
    userTokenAccount = (await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      mint,
      user.publicKey
    )).address;

    // Mint tokens to user
    await mintTo(
      provider.connection,
      creator,
      mint,
      userTokenAccount,
      creator,
      1000_000_000 // Mint 1000 tokens
    );

    // Find PDA for market
    [marketPda, marketBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("market"), Buffer.from(marketName)],
      program.programId
    );

    // Create market token account
    const marketTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      creator,
      mint,
      marketPda,
      true
    );
    marketTokenAccount = marketTokenAccountInfo.address;

    // Find PDA for bet
    [betPda, betBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("bet"), marketPda.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes the market", async () => {
    const expiryTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now

    await program.methods
      .initializeMarket(marketName, new anchor.BN(expiryTimestamp))
      .accounts({
        market: marketPda,
        creator: creator.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([creator])
      .rpc();

    const marketAccount = await program.account.predictionMarket.fetch(marketPda);
    assert.equal(marketAccount.name, marketName);
    assert.equal(marketAccount.creator.toBase58(), creator.publicKey.toBase58());
    assert.equal(marketAccount.expiryTimestamp.toNumber(), expiryTimestamp);
    assert.equal(marketAccount.settled, false);
  });

  it("Places a bet", async () => {
    await program.methods
      .placeBet(betAmount, prediction)
      .accounts({
        market: marketPda,
        bet: betPda,
        user: user.publicKey,
        userTokenAccount: userTokenAccount,
        marketTokenAccount: marketTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const betAccount = await program.account.bet.fetch(betPda);
    assert.equal(betAccount.user.toBase58(), user.publicKey.toBase58());
    assert.equal(betAccount.market.toBase58(), marketPda.toBase58());
    assert.equal(betAccount.amount.toNumber(), betAmount.toNumber());
    assert.equal(betAccount.prediction, prediction);

    const marketAccount = await program.account.predictionMarket.fetch(marketPda);
    assert.equal(marketAccount.yesAmount.toNumber(), betAmount.toNumber());
    assert.equal(marketAccount.noAmount.toNumber(), 0);
  });

  it("Fails to place a bet after expiry", async () => {
    // Wait for the market to expire
    await new Promise((resolve) => setTimeout(resolve, 3600 * 1000));

    try {
      await program.methods
        .placeBet(betAmount, prediction)
        .accounts({
          market: marketPda,
          bet: betPda,
          user: user.publicKey,
          userTokenAccount: userTokenAccount,
          marketTokenAccount: marketTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user])
        .rpc();
      assert.fail("Expected an error but none was thrown");
    } catch (error) {
      assert.include(error.message, "Market has expired");
    }
  });

  it("Settles the market", async () => {
    const outcome = true; // Price went up

    await program.methods
      .settleMarket(outcome)
      .accounts({
        market: marketPda,
        creator: creator.publicKey,
      })
      .signers([creator])
      .rpc();

    const marketAccount = await program.account.predictionMarket.fetch(marketPda);
    assert.equal(marketAccount.outcome, outcome);
    assert.equal(marketAccount.settled, true);
  });

  it("Claims winnings", async () => {
    const initialUserBalance = await provider.connection.getTokenAccountBalance(userTokenAccount);

    await program.methods
      .claimWinnings()
      .accounts({
        market: marketPda,
        bet: betPda,
        user: user.publicKey,
        userTokenAccount: userTokenAccount,
        marketTokenAccount: marketTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const finalUserBalance = await provider.connection.getTokenAccountBalance(userTokenAccount);
    assert.isTrue(new anchor.BN(finalUserBalance.value.amount).gt(new anchor.BN(initialUserBalance.value.amount)));
  });

  it("Fails to claim winnings twice", async () => {
    try {
      await program.methods
        .claimWinnings()
        .accounts({
          market: marketPda,
          bet: betPda,
          user: user.publicKey,
          userTokenAccount: userTokenAccount,
          marketTokenAccount: marketTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      assert.fail("Expected an error but none was thrown");
    } catch (error) {
      assert.include(error.message, "You are not a winner in this market");
    }
  });
});