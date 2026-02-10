"""
Test utilities and fixtures for MCP client tests

Provides common mock setups to reduce code duplication.
"""

import pytest
from unittest.mock import MagicMock, patch
from mcp_client import McpClient


@pytest.fixture
def mock_subprocess_process():
    """
    Fixture providing a mocked subprocess.Popen process with standard setup.

    Returns:
        MagicMock: Configured mock process with stdin, stdout, stderr
    """
    process = MagicMock()
    process.stdin = MagicMock()
    process.stdout = MagicMock()
    process.stderr = MagicMock()
    return process


@pytest.fixture
def mock_subprocess_process_with_wait(mock_subprocess_process):
    """
    Fixture providing a mocked process with wait() return value.

    Returns:
        MagicMock: Process with wait() configured to return 0
    """
    mock_subprocess_process.wait = MagicMock(return_value=0)
    return mock_subprocess_process


@pytest.fixture
def mock_popen_factory(mock_subprocess_process):
    """
    Fixture factory for creating mock subprocess.Popen contexts.

    Yields:
        MagicMock: Mocked subprocess.Popen configured to return mock_process
    """
    with patch("subprocess.Popen") as mock_popen:
        mock_popen.return_value = mock_subprocess_process
        yield mock_popen


@pytest.fixture
def mock_spawned_client(mock_popen_factory):
    """
    Fixture providing a client that has already been spawned.

    This is the most common setup pattern for lifecycle tests.

    Yields:
        tuple: (client, mock_process, mock_popen) where client is spawned
               and ready for initialize/shutdown tests
    """
    client = McpClient("test", ["echo", "test"])
    client.spawn()

    # Get the mock process that was created
    mock_process = mock_popen_factory.return_value
    mock_popen = mock_popen_factory

    yield client, mock_process, mock_popen


@pytest.fixture
def mock_initialized_client(mock_spawned_client):
    """
    Fixture providing a client that is spawned and initialized.

    This is the most common setup pattern for tool operation tests.

    Yields:
        tuple: (client, mock_send) where client is ready for tool operations
    """
    client, mock_process, mock_popen = mock_spawned_client

    # Mock the _send_request method for initialize
    with patch.object(client, "_send_request") as mock_send:
        mock_send.return_value = {"result": {"protocolVersion": "2024-11-05"}}
        client.initialize()

        yield client, mock_send
