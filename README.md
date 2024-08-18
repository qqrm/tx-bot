# Transaction Bot

## Overview

This Rust project implements a bot that processes transactions by sending them from a specified wallet to purchase a specific token. The bot continues to send transactions until it either spends the specified total amount or reaches the maximum number of transactions, as defined in an `.env` file.

The bot supports multi-threaded execution, with the number of concurrent threads configurable via the `.env` file. Each transaction's commission varies within a range defined in the configuration.

## Summary

It seems like I might not have covered everything, but it looks like enough has been done for a toy project. The tests and error handling could be improved. It also seems like using tokio and async might make sense, but that would contradict the task's requirements. Spend ~4-5hrs.