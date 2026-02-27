from __future__ import annotations

from typing import Awaitable


class OperationDefinition:
    """Metadata for one generated `OpenAPI` operation.
    
    Returned by `Client.operations()` and `AsyncClient.operations()`.
    """
    @property
    def operation_id(self) -> str:
        """Stable `OpenAPI` operation identifier (for example, `"getFacility"`)."""
        ...

    @property
    def method(self) -> str:
        """Uppercase HTTP method (for example, `"GET"` or `"POST"`)."""
        ...

    @property
    def path_template(self) -> str:
        """Path template which may contain `{param}` placeholders."""
        ...

    @property
    def path_params(self) -> list[str]:
        """Required path-parameter names extracted from `path_template`."""
        ...


class Client:
    """Synchronous Python client for the IRI API.
    
    This class wraps the blocking Rust client and returns JSON payloads as
    strings.
    """
    def __init__(
        self,
        base_url: str | None = None,
        access_token: str | None = None,
    ) -> None: ...
    @staticmethod
    def operations() -> list[OperationDefinition]:
        """Return all generated `OpenAPI` operation definitions."""
        ...
    def get(self, path: str) -> str:
        """Perform a `GET` request against a raw API path."""
        ...
    def request(
        self,
        method: str,
        path: str,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> str:
        """Perform a raw HTTP request by method and path.
        
        Args:
            method: HTTP method (for example, `"GET"`).
            path: API path relative to the configured base URL.
            `query_json`: Optional JSON object (string) used as query parameters.
            `body_json`: Optional JSON value (string) used as request body.
        
        Returns:
            A JSON string containing the parsed response payload.
        """
        ...
    def call_operation(
        self,
        operation_id: str,
        path_params_json: str | None = None,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> str:
        """Call an endpoint by `OpenAPI` `operation_id`.
        
        Args:
            `operation_id`: Operation identifier from `Client.operations()`.
            `path_params_json`: Optional JSON object (string) for path parameters.
            `query_json`: Optional JSON object (string) for query parameters.
            `body_json`: Optional JSON value (string) for request body.
        
        Returns:
            A JSON string containing the parsed response payload.
        """
        ...


class AsyncClient:
    """Asynchronous Python client for the IRI API.
    
    Methods return awaitables using the Tokio runtime integration from
    `pyo3-async-runtimes`.
    """
    def __init__(
        self,
        base_url: str | None = None,
        access_token: str | None = None,
    ) -> None: ...
    @staticmethod
    def operations() -> list[OperationDefinition]:
        """Return all generated `OpenAPI` operation definitions."""
        ...
    def get(self, path: str) -> Awaitable[str]:
        """Perform an asynchronous `GET` request against a raw API path."""
        ...
    def request(
        self,
        method: str,
        path: str,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> Awaitable[str]:
        """Perform an asynchronous raw HTTP request by method and path.
        
        Args:
            method: HTTP method (for example, `"GET"`).
            path: API path relative to the configured base URL.
            `query_json`: Optional JSON object (string) used as query parameters.
            `body_json`: Optional JSON value (string) used as request body.
        
        Returns:
            An awaitable resolving to a JSON string response payload.
        """
        ...
    def call_operation(
        self,
        operation_id: str,
        path_params_json: str | None = None,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> Awaitable[str]:
        """Call an endpoint asynchronously by `OpenAPI` `operation_id`.
        
        Args:
            `operation_id`: Operation identifier from `AsyncClient.operations()`.
            `path_params_json`: Optional JSON object (string) for path parameters.
            `query_json`: Optional JSON object (string) for query parameters.
            `body_json`: Optional JSON value (string) for request body.
        
        Returns:
            An awaitable resolving to a JSON string response payload.
        """
        ...
