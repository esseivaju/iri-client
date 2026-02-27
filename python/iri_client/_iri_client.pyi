from __future__ import annotations

from typing import Awaitable


class OperationDefinition:
    @property
    def operation_id(self) -> str: ...
    @property
    def method(self) -> str: ...
    @property
    def path_template(self) -> str: ...
    @property
    def path_params(self) -> list[str]: ...


class Client:
    def __init__(
        self,
        base_url: str | None = None,
        access_token: str | None = None,
    ) -> None: ...
    @staticmethod
    def operations() -> list[OperationDefinition]: ...
    def get(self, path: str) -> str: ...
    def request(
        self,
        method: str,
        path: str,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> str: ...
    def call_operation(
        self,
        operation_id: str,
        path_params_json: str | None = None,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> str: ...


class AsyncClient:
    def __init__(
        self,
        base_url: str | None = None,
        access_token: str | None = None,
    ) -> None: ...
    @staticmethod
    def operations() -> list[OperationDefinition]: ...
    def get(self, path: str) -> Awaitable[str]: ...
    def request(
        self,
        method: str,
        path: str,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> Awaitable[str]: ...
    def call_operation(
        self,
        operation_id: str,
        path_params_json: str | None = None,
        query_json: str | None = None,
        body_json: str | None = None,
    ) -> Awaitable[str]: ...
