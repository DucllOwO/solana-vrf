import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Random } from "../target/types/random";
import { Big } from "@switchboard-xyz/common";
import { AggregatorAccount, AnchorWallet, SwitchboardProgram, SwitchboardTestContext, Callback, PermissionAccount, QueueAccount, SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_QUEUE, VrfAccount } from "@switchboard-xyz/solana.js"
import { assert } from "chai";
import { OracleQueueAccountData } from "@switchboard-xyz/solana.js/lib/generated";
import { StatusVerified } from "@switchboard-xyz/solana.js/lib/generated/oracle-program/types/VrfStatus";
import MerkleTree from "merkletreejs";
import sha256 from 'crypto-js/sha256';
import { BN } from "bn.js";
import { ComputeBudgetProgram } from "@solana/web3.js";

function delay(ms: number) {
  return new Promise( resolve => setTimeout(resolve, ms) );
}

describe.only("random-vrf", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env()
  console.log("🚀 ~ describe.only ~ provider:", provider.connection.rpcEndpoint)
  const program = anchor.workspace.Random as Program<Random>;
  const payer = (provider.wallet as AnchorWallet).payer

  // ADDED CODE
  let switchboardProgram: SwitchboardProgram;
  let queueAccount: QueueAccount;
  let switchboard: SwitchboardTestContext | undefined;
  let queue: OracleQueueAccountData;

  before(async () => {
    console.log(`random programId: ${program.programId}`);
    switchboardProgram = await SwitchboardProgram.fromProvider(provider);
      if (await switchboardProgram.cluster === "devnet") {
        queueAccount = new QueueAccount(
          switchboardProgram,
          SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_QUEUE
        );
      }  else {
        throw new Error(
          `Failed to load Switchboard queue for cluster, ${switchboardProgram.cluster}`
        );
      }
      queue = await queueAccount.loadData();
  })

  it("request randomness", async () => {
    let bytes = 8 + 1 + 64 + 4 + 4 + 32 + 8 + 32 + 4 + 4*750;
    let temp = await provider.connection.getMinimumBalanceForRentExemption(40289, "max");

    console.log(temp)
    let roundId = 1205;
    const roundSecret = anchor.web3.Keypair.generate()
    const [roundState, roundBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("Round"),
        payer.publicKey.toBytes(),
        roundSecret.publicKey.toBytes(),
        Buffer.from(roundId.toString())
      ],
      program.programId
    )
    console.log(`Round state: ${roundState}`)

    const randomIxCoder = new anchor.BorshInstructionCoder(program.idl)
    const roundCallback: Callback = {
      programId: program.programId,
      accounts: [
        { pubkey: roundState, isSigner: false, isWritable: true },
        { pubkey: roundSecret.publicKey, isSigner: false, isWritable: true },
      ],
      ixData: randomIxCoder.encode("consumeRandomness", ""), // pass any params for instruction here
    }
    // Create Switchboard VRF and Permission account
    const [vrfAccount, vrfInitTxn] = await queueAccount.createVrfInstructions(
      payer.publicKey,
      {
        callback: roundCallback,
        authority: roundState, // vrf authority
        vrfKeypair: roundSecret,
        enable: false, // only set permissions if required
      }
    );

    await switchboardProgram.signAndSend(vrfInitTxn, {
      preflightCommitment: "max",
    });

    // vrf data
    let vrf = await vrfAccount.loadData();
    console.log(vrf.authority);


    const [payerTokenWallet] = await switchboardProgram.mint.getOrCreateWrappedUser(
      switchboardProgram.walletPubkey,
      { fundUpTo: 0.1 }
    );
    console.log("🚀 ~ it ~ payerTokenWallet:", payerTokenWallet)
    
    // derive the existing VRF permission account using the seeds
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      switchboardProgram,
      queue.authority,
      queueAccount.publicKey,
      vrfAccount.publicKey
    )

    for (let index = 0; index < 1; index++) {
      try {
        const prizeCount = 749;
        const leaves = ['0,68058', '1,68057', '42058', '65058', '68757', '22058'].map(x => Buffer.from(sha256(x).toString(), 'hex'))
        const tree = new MerkleTree(leaves, sha256)
        const root = tree.getRoot()
        // Request randomness and roll dice
        const requestInstruction = await program.methods.requestRandomness(Buffer.from(roundId.toString()), {
          switchboardStateBump: switchboardProgram.programState.bump,
          permissionBump,
          nftCount: 10000,
          merkleRoot: root,
          roundBump: roundBump,
          prizeCount: prizeCount
        })
        .accounts({
          roundState: roundState,
          vrf: vrfAccount.publicKey,
          user: payer.publicKey,
          payerWallet: payerTokenWallet,
          oracleQueue: queueAccount.publicKey,
          queueAuthority: queue.authority,
          dataBuffer: queue.dataBuffer,
          permission: permissionAccount.publicKey,
          switchboardEscrow: vrf.escrow,
          programState: switchboardProgram.programState.publicKey,
          switchboardProgram: new anchor.web3.PublicKey("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f"), //switchboardProgram.attestationProgramId,
          recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([payer])
        .instruction()

        const transaction = new anchor.web3.Transaction()
        .add(requestInstruction);
        
        const tx = await program.provider.sendAndConfirm(transaction)
        
        console.log("tx: ", tx);
        
        await provider.connection.confirmTransaction(tx, "confirmed")
  
        console.log(`round state: ${roundState}`)
  
        console.log("Rolling result...")
  
        let didUpdate = false;
        let roundCurrentState = await program.account.roundState.fetch(roundState)
        let result = [];
        while (true) {
          console.log("Checking result...")
          roundCurrentState = await program.account.roundState.fetch(roundState);
          didUpdate = roundCurrentState.timestamp.toNumber() > 0;
          vrf = await vrfAccount.loadData();
          if (vrf.status.kind === "StatusCallbackSuccess" && didUpdate) {
            console.time("start get result")
            while (true) {
              // get random number 
              let drawRandom = await program.methods.drawRandomNumber(50).accounts({ roundState: roundState }).signers([payer]).instruction()

              const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
                units: 1000000
              });

              const transaction = new anchor.web3.Transaction()
                .add(modifyComputeUnits, drawRandom);
              
              const tx = await program.provider.sendAndConfirm(transaction)
              
            
              await provider.connection.confirmTransaction(tx, "max")
              
              roundCurrentState = await program.account.roundState.fetch(roundState);
              console.log("draw tx hash: ",tx)

              console.log("current prize remaning prize: ", roundCurrentState.prizeRemaining);

              if (roundCurrentState.prizeRemaining <= 0) {
                break
              }
            }
          } else {
            console.log("Wating vrf status to success 2 seconds....");
            await delay(2000)
          }
          roundCurrentState = await program.account.roundState.fetch(roundState);
          if (roundCurrentState.prizeRemaining <= 0) {
            console.log(roundCurrentState.winner_indexes)
            break
          }
        }
      } catch (e) {
        console.log(e)
        assert.fail()
      }
      
    }
  })
});