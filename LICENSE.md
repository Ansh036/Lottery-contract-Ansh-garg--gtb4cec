Project Title
A brief description of what this project does and who it's for

Soroban Lottery Contract
This contract provides lottery implementation for the Soroban smart contract platform. It allows an administrator to create and manage lotteries where participants can buy tickets and win prizes based on correctly selected numbers.

Features
Lottery Initialization: Admin can specify parameters such as the number of numbers to be drawn, the range of numbers, thresholds for prize distribution, and minimum participant count.
Ticket Purchase: Participants can buy tickets by selecting numbers within the specified range.
Prize Distribution: Prizes are distributed based on the number of correctly selected numbers and the defined thresholds.
Handling Multiple Lotteries: The contract supports multiple lotteries, allowing for sequential draws.
Usage
Initialization
The contract must be initialized by the admin using the init function. This sets up the parameters for the lottery.

Creating a Lottery
After initialization, the admin can create a new lottery using the create_lottery function. This clears previous tickets and sets up a new lottery with specified parameters.

Ticket Purchase
Participants can buy tickets by calling the buy_ticket function with their selected numbers. They must have sufficient funds to purchase a ticket.

Checking Results
Participants can check the results of a specific lottery by calling the check_lottery_results function with the lottery number.

Playing the Lottery
The admin can play the lottery using the play_lottery function, which randomly selects winning numbers, calculates prizes, and distributes them to the winners.

Error Handling
The contract defines various error types to handle scenarios such as insufficient funds, invalid parameters, and unauthorized actions.

Contract Structure
LotteryState: Enum representing the state of the lottery (Initialized, Active, Finished).
DataKey: Enum defining storage keys used by the contract.
Error: Enum defining contract errors.
LotteryContract: Main contract structure.
get_winners: Function to calculate winners based on selected numbers and thresholds.
calculate_prizes: Function to calculate prizes for winners.
payout_prizes: Function to distribute prizes to winning participants.
draw_numbers: Function to randomly draw numbers for a lottery.
test: Module containing unit tests for the contract.
Testing
The contract includes a comprehensive test suite to ensure functionality and error handling are working as expected.

Development Setup
To set up a development environment for this contract, follow the instructions provided in the Soroban documentation for contract deployment and testing.

Contributing
Contributions to this contract are welcome. If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

License
This project is licensed under the MIT License. See the LICENSE file for details
