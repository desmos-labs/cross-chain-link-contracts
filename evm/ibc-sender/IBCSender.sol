// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../interfaces/IWormhole.sol";

contract IBCSender {
    uint256 constant GAS_LIMIT = 50_000;
    IWormhole public immutable wormhole;

    constructor(address _wormhole) {
        wormhole = IWormhole(_wormhole);
    }

    function sendIBCPacket(
        string memory payload
    ) public payable {
        wormhole.publishMessage{
            value: wormhole.messageFee()
        }(1,  bytes(payload), 200);
    }
}