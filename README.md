# ChatLogger

ChatLogger is a Python package written in RUST for managing chat conversations with support for branching dialogue, user management, and integration with language models.

## Features

- Manage multiple chat conversations
- Support for branching dialogue
- User management system
- Flask-based web [interface](https://github.com/thefcraft/chatlogger/tree/master/example-llmui) example for interacting with chats

## Installation

```bash
pip install chatlogger-db
```

## Documentation

Real developers learn from examples [example-llmui](https://github.com/thefcraft/chatlogger/tree/master/example-llmui)

## Quick Start

Here's a basic example of how to use ChatLogger:

```python
from chatlogger import DataBase, Chat, Memory, UserTable

# Initialize the database and user table
db = DataBase("database.bin")
user_table = UserTable(db, "user_table.bin")

# Create a new user
user_table.new_user(userid="userid1", username="laskh")
user = user_table["userid1"]

# Create a new chat
chat_uuid = user.new_chat(prompt_info="Python chatlogger app...")
chat, info, timestamp = user[chat_uuid]

# Add messages to the chat
chat.new_message("write code for Python chatlogger app.", "OK! Tell me more about it so I assist you.")

# Save changes
db.commit()
user_table.commit()
```

## Demo Web Interface

[example-llmui](https://github.com/thefcraft/chatlogger/tree/master/example-llmui) includes a Flask-based web interface for interacting with chats. To run the web interface:

`python app.py`

![Web Interface Screenshot](/example-llmui/img.png)



## Core Components

### DataBase

The `DataBase` class manages the storage and retrieval of chat data.

### Chat

The `Chat` class represents a single chat conversation, allowing for branching dialogue.

### Memory

The `Memory` class handles the formatting of chat history for use with language models.

### UserTable

The `UserTable` class manages user data and their associated chats.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/git/git-scm.com/blob/main/MIT-LICENSE.txt) file for details.
