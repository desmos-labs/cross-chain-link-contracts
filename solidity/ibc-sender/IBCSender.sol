// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "wormhole-solidity-sdk/interfaces/IWormhole.sol";

contract IBCSender {
    uint256 constant GAS_LIMIT = 50_000;
    IWormhole public immutable wormhole;

    constructor(address _wormhole) {
        wormhole = IWormhole(_wormhole);
    }

    function sendIBCPacket(
        uint16 targetChain,
        address targetAddress,
        string memory payload
    ) public payable {
        wormhole.publishMessage{
            value: wormhole.messageFee()
        }(1, payload, 200);
    }
}