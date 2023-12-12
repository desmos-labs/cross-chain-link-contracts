import dotenv from 'dotenv';
import { Wormhole } from '@wormhole-foundation/connect-sdk';
import { EvmPlatform } from '@wormhole-foundation/connect-sdk-evm';
import { CosmwasmPlatform } from '@wormhole-foundation/connect-sdk-cosmwasm';

import Web3 from 'web3';
import { ContractABI, ContractABIType, LogMessagePublishedABI } from './evm';
import Wallet from 'ethereumjs-wallet';
import { toBuffer } from 'ethereumjs-util';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { makeSignDoc, decodeSignature, Pubkey, pubkeyType, serializeSignDoc } from '@cosmjs/amino';

import { OfflineSignerAdapter, SigningMode } from '@desmoslabs/desmjs';
import { Profiles } from '@desmoslabs/desmjs';
import { SingleSignature, SignatureValueType, HexAddress, ChainConfig } from '@desmoslabs/desmjs-types/desmos/profiles/v3/models_chain_links';

dotenv.config();

const { EVM_PRIVATE_KEY, EVM_RPC, EVM_CONTRACT, COSMWASM_GATEWAY_MNEMONIC, COSMWASM_PREFIX, COSMWASM_RPC, COSMWASM_CONTRACT, IBC_CHANNEL_ID, DESMOS_MNEMONIC } = process.env;

async function main() {
    // Setup environment
    const web3 = new Web3(new Web3.providers.HttpProvider(EVM_RPC!))
    const wh = new Wormhole("Testnet", [EvmPlatform, CosmwasmPlatform]);

    // EVM environment
    const evmContractAddress = wh.parseAddress("Polygon", EVM_CONTRACT!); // contract on Polygon Mumbai
    const evmAccount = web3.eth.accounts.privateKeyToAccount("0x" + EVM_PRIVATE_KEY!);
    web3.eth.accounts.wallet.add(evmAccount);
    const evmContract = new web3.eth.Contract(ContractABI as ContractABIType, evmContractAddress.toString(), { from: evmAccount.address })
    const evmPubkeyHex = Wallet.fromPrivateKey(toBuffer("0x" + EVM_PRIVATE_KEY!)).getPublicKeyString();

    // Cosmwasm environment
    const relayerSigner = await DirectSecp256k1HdWallet.fromMnemonic(COSMWASM_GATEWAY_MNEMONIC!, { prefix: COSMWASM_PREFIX! });
    const [relayerAccount] = await relayerSigner.getAccounts();
    const relayerClient = await SigningCosmWasmClient.connectWithSigner(COSMWASM_RPC!, relayerSigner, { gasPrice: GasPrice.fromString("0.03uluna") });

    // Desmos environment
    const desmosSigner = await OfflineSignerAdapter.fromMnemonic(SigningMode.AMINO, DESMOS_MNEMONIC!);
    const [desmosAccount] = await desmosSigner.getAccounts();

    // Prepare payload with IBC packet
    const { signed: destinationPlainText, signature: destinationSignature } = await desmosSigner.signAmino(
        desmosAccount.address,
        makeSignDoc([], { amount: [], gas: "0" }, "morpheus-apollo-3", evmAccount.address, 0, 0),
    );
    const decodedDestinationSig = decodeSignature(destinationSignature);
    const destinationPlainTextHex = Buffer.from(serializeSignDoc(destinationPlainText)).toString("hex");

    const sourceSignatureData = await evmAccount.sign(desmosAccount.address);
    const { signature: sourceSignature } = sourceSignatureData;
    const evmPubKey: Pubkey = {
        type: pubkeyType.secp256k1,
        value: Buffer.concat([Buffer.from([0x4]), Buffer.from(evmPubkeyHex.replace("0x", ""), "hex")]).toString("base64"),
    };

    const sourcePlainText = "\x19Ethereum Signed Message:\n" + desmosAccount.address.length + desmosAccount.address;
    const packet = generatePacket(
        ChainConfig.fromPartial({ name: "polygon" }),
        HexAddress.fromPartial({
            prefix: "0x",
            value: evmAccount.address,
        }),
        evmPubKey,
        SingleSignature.fromPartial({
            valueType: SignatureValueType.SIGNATURE_VALUE_TYPE_EVM_PERSONAL_SIGN,
            signature: Uint8Array.from(Buffer.from(sourceSignature.replace("0x", ""), 'hex')),
        }),
        Buffer.from(sourcePlainText).toString("hex"),
        desmosAccount.address,
        destinationSignature.pub_key,
        SingleSignature.fromPartial({
            valueType: SignatureValueType.SIGNATURE_VALUE_TYPE_COSMOS_AMINO,
            signature: decodedDestinationSig.signature,
        }),
        destinationPlainTextHex,
    );
    const payload = {
        channel_id: IBC_CHANNEL_ID!,
        packet: Buffer.from(JSON.stringify(packet)).toString("base64"),
    };

    // Send payload with IBC packet to evm chain
    console.log("Execute evm contract with IBC packet...")
    let published = await evmContract.methods.sendIBCPacket(JSON.stringify(payload)).send();
    const { sequence } = web3.eth.abi.decodeLog(LogMessagePublishedABI, published.logs[0].data!, published.logs[1].topics!);
    console.log(`Payload published to Wormhole, tx: ${published.transactionHash}`)

    // Retrieves Vaa bytes
    console.log("Get Vaa from Guardian Network...");
    const vaa = await wh.getVAABytes("Polygon", evmContractAddress, BigInt(sequence as number));
    const vaaBase64 = Buffer.from(vaa!).toString("base64");

    // Relay VAA to Cosmwasm gateway chain
    console.log("Submit Vaa to gateway chain...");
    const result = await relayerClient.execute(relayerAccount.address, COSMWASM_CONTRACT!, { submit_vaa: { data: vaaBase64 } }, "auto");
    console.log(`IBC packet to Desmos relayed, tx: ${result.transactionHash}`);
    console.log("Finished");
}

function generatePacket(
    sourceChainConfig: ChainConfig,
    sourceAddress: HexAddress,
    sourcePubkey: Pubkey,
    sourceSig: SingleSignature,
    sourcePlainTextHex: string,
    destinationAddress: string,
    destinationPubkey: Pubkey,
    destinationSig: SingleSignature,
    destinationPlainTextHex: string,
): unknown {
    return {
        sourceChainConfig,
        sourceAddress: {
            "@type": Profiles.v3.HexAddressTypeUrl,
            ...sourceAddress,
        },
        sourceProof: {
            pubKey: {
                "@type": "/ethermint.crypto.v1.ethsecp256k1.PubKey",
                key: sourcePubkey.value,
            },
            plainText: sourcePlainTextHex,
            signature: {
                "@type": Profiles.v3.SingleSignatureTypeUrl,
                valueType: "SIGNATURE_VALUE_TYPE_EVM_PERSONAL_SIGN",
                signature: Buffer.from(sourceSig.signature).toString("base64")
            },
        },
        destinationProof: {
            pubKey: {
                "@type": "/cosmos.crypto.secp256k1.PubKey",
                key: destinationPubkey.value,
            },
            plainText: destinationPlainTextHex,
            signature: {
                "@type": Profiles.v3.SingleSignatureTypeUrl,
                valueType: "SIGNATURE_VALUE_TYPE_COSMOS_AMINO",
                signature: Buffer.from(destinationSig.signature).toString("base64")
            }
        },
        destinationAddress,
    }
}

main().catch(err => console.error(err));
