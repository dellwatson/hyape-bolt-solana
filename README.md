2 Component Contracts:

RoomComponent: Stores room data including player list
PlayerComponent: Stores all player data including position, rotation, animation

3-5 System Contracts:

CreateRoomSystem: Creates a room and initial player
JoinRoomSystem: Allows additional players to join a room
UpdatePlayerSystem: Updates player position, rotation, animation
StartGameSystem: Changes room status to in-progress
FinalizeGameSystem: Ends the game and handles cleanup
