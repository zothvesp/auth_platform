"use client";

import {
  Box,
  Checkbox,
  Group,
  Menu,
  Pagination,
  ScrollArea,
  Select,
  Stack,
  Table,
  Text,
  TextInput,
} from "@mantine/core";
import { IconChevronDown, IconColumns } from "@tabler/icons-react";
import {
  type ColumnDef,
  type ColumnFiltersState,
  type PaginationState,
  type RowSelectionState,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useMemo, useState, type ReactNode } from "react";
import { useTranslations } from "@lib/i18n";
import { AppButton } from "./button";
import { BulkActionBar } from "./bulk-action-bar";
import { EmptyState } from "./state";
import { SearchInput } from "./search-input";
import { SurfaceCard } from "./surface-card";
import { TableColumnHeader } from "./table-column-header";

export type DataTableProps<TData extends object> = {
  columns: ColumnDef<TData, any>[];
  data: TData[];
  emptyText?: string;
  enableColumnFilters?: boolean;
  enableColumnVisibility?: boolean;
  enableGlobalFilter?: boolean;
  enablePagination?: boolean;
  enableRowSelection?: boolean;
  getRowId?: (row: TData, index: number) => string;
  initialPageSize?: number;
  renderBulkActions?: (selectedRows: TData[]) => ReactNode;
  renderRowActions?: (row: TData) => ReactNode;
  toolbarLeft?: ReactNode;
  toolbarRight?: ReactNode;
};

const pageSizeOptions = ["10", "20", "30", "50", "100"];

export function DataTable<TData extends object>({
  columns,
  data,
  emptyText,
  enableColumnFilters = true,
  enableColumnVisibility = true,
  enableGlobalFilter = true,
  enablePagination = true,
  enableRowSelection = false,
  getRowId,
  initialPageSize = 10,
  renderBulkActions,
  renderRowActions,
  toolbarLeft,
  toolbarRight,
}: DataTableProps<TData>) {
  const t = useTranslations();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [globalFilter, setGlobalFilter] = useState("");
  const [rowSelection, setRowSelection] = useState<RowSelectionState>({});
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: initialPageSize,
  });

  const tableColumns = useMemo<ColumnDef<TData, any>[]>(() => {
    const selectionColumn: ColumnDef<TData, any>[] = enableRowSelection
      ? [
          {
            id: "__select",
            enableHiding: false,
            enableSorting: false,
            header: ({ table }) => (
              <Checkbox
                aria-label={t.common.selectAllRows}
                checked={table.getIsAllPageRowsSelected()}
                indeterminate={table.getIsSomePageRowsSelected()}
                onChange={table.getToggleAllPageRowsSelectedHandler()}
              />
            ),
            cell: ({ row }) => (
              <Checkbox
                aria-label={t.common.selectRow}
                checked={row.getIsSelected()}
                disabled={!row.getCanSelect()}
                indeterminate={row.getIsSomeSelected()}
                onChange={row.getToggleSelectedHandler()}
              />
            ),
          },
        ]
      : [];

    const actionColumn: ColumnDef<TData, any>[] = renderRowActions
      ? [
          {
            id: "__actions",
            header: "",
            enableHiding: false,
            enableSorting: false,
            cell: ({ row }) => (
              <Group position="right" spacing={4} noWrap>
                {renderRowActions(row.original)}
              </Group>
            ),
          },
        ]
      : [];

    return [...selectionColumn, ...columns, ...actionColumn];
  }, [columns, enableRowSelection, renderRowActions, t]);

  const table = useReactTable({
    data,
    columns: tableColumns,
    state: {
      sorting,
      columnFilters,
      columnVisibility,
      globalFilter,
      rowSelection,
      pagination,
    },
    enableRowSelection,
    getRowId,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onColumnVisibilityChange: setColumnVisibility,
    onGlobalFilterChange: setGlobalFilter,
    onRowSelectionChange: setRowSelection,
    onPaginationChange: setPagination,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
  });

  const selectedRows = table.getSelectedRowModel().flatRows.map((row) => row.original);
  const visibleColumnCount = table.getVisibleLeafColumns().length;
  const rows = enablePagination ? table.getRowModel().rows : table.getPrePaginationRowModel().rows;
  const hasToolbar =
    toolbarLeft ||
    toolbarRight ||
    enableGlobalFilter ||
    enableColumnVisibility ||
    (enableRowSelection && selectedRows.length > 0);

  return (
    <SurfaceCard>
      {hasToolbar ? (
        <Box px="md" py="sm" sx={(theme) => ({ borderBottom: `1px solid ${theme.colors.dark[4]}` })}>
          <Group position="apart" align="center">
            <Group spacing="xs">
              {toolbarLeft}
              {enableGlobalFilter ? (
                <SearchInput
                  placeholder={t.common.search}
                  value={globalFilter ?? ""}
                  onChange={setGlobalFilter}
                  w={260}
                />
              ) : null}
            </Group>

            <Group spacing="xs">
              {enableRowSelection && selectedRows.length > 0 ? (
                <BulkActionBar
                  selectedCount={selectedRows.length}
                  actions={renderBulkActions?.(selectedRows)}
                  onClear={() => table.resetRowSelection()}
                />
              ) : null}

              {toolbarRight}

              {enableColumnVisibility ? (
                <Menu shadow="md" width={220} position="bottom-end">
                  <Menu.Target>
                    <AppButton
                      appVariant="secondary"
                      icon={<IconColumns size={15} />}
                      rightIcon={<IconChevronDown size={14} />}
                    >
                      {t.common.columns}
                    </AppButton>
                  </Menu.Target>
                  <Menu.Dropdown>
                    {table
                      .getAllLeafColumns()
                      .filter((column) => column.getCanHide())
                      .map((column) => (
                        <Menu.Item key={column.id} closeMenuOnClick={false}>
                          <Checkbox
                            label={column.columnDef.header?.toString() || column.id}
                            checked={column.getIsVisible()}
                            onChange={column.getToggleVisibilityHandler()}
                          />
                        </Menu.Item>
                      ))}
                  </Menu.Dropdown>
                </Menu>
              ) : null}
            </Group>
          </Group>
        </Box>
      ) : null}

      <ScrollArea>
        <Table striped highlightOnHover verticalSpacing="sm" fontSize="sm">
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr className="uir-list-headerrow" key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  const sortable = header.column.getCanSort();
                  const sortDirection = header.column.getIsSorted();

                  return (
                    <th
                      key={header.id}
                      style={{
                        width: header.getSize(),
                        whiteSpace: "nowrap",
                      }}
                    >
                      {header.isPlaceholder ? null : (
                        <Stack spacing={6}>
                          <TableColumnHeader
                            label={flexRender(header.column.columnDef.header, header.getContext())}
                            sortable={sortable}
                            sorted={sortDirection}
                            onSort={() => header.column.toggleSorting()}
                          />

                          {enableColumnFilters && header.column.getCanFilter() ? (
                            <TextInput
                              aria-label={`Filter ${header.column.id}`}
                              size="xs"
                              placeholder="Filter"
                              value={(header.column.getFilterValue() ?? "") as string}
                              onChange={(event) =>
                                header.column.setFilterValue(event.currentTarget.value)
                              }
                            />
                          ) : null}
                        </Stack>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {rows.length ? (
              rows.map((row) => (
                <tr
                  key={row.id}
                  className={row.getIsSelected() ? "uir-machine-row-focused" : "uir-machine-row"}
                >
                  {row.getVisibleCells().map((cell) => (
                    <td
                      key={cell.id}
                      style={{
                        textAlign: cell.column.id === "__actions" ? "right" : undefined,
                      }}
                    >
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))
            ) : (
              <tr>
                <td colSpan={visibleColumnCount}>
                  <EmptyState title={emptyText ?? t.common.noRecordsFound} description={t.common.noMatchingRecords} />
                </td>
              </tr>
            )}
          </tbody>
        </Table>
      </ScrollArea>

      {enablePagination ? (
        <Box px="md" py="sm" sx={(theme) => ({ borderTop: `1px solid ${theme.colors.dark[4]}` })}>
          <Group position="apart">
            <Text size="xs" color="dimmed">
              {t.common.showingOf(rows.length, table.getFilteredRowModel().rows.length)}
            </Text>
            <Group spacing="xs">
              <Select
                aria-label="Rows per page"
                size="xs"
                w={118}
                value={String(table.getState().pagination.pageSize)}
                onChange={(value) => table.setPageSize(Number(value ?? initialPageSize))}
                  data={pageSizeOptions.map((value) => ({
                    value,
                    label: t.common.perPage(Number(value)),
                  }))}
              />
              <Pagination
                size="sm"
                page={table.getState().pagination.pageIndex + 1}
                total={Math.max(table.getPageCount(), 1)}
                onChange={(page) => table.setPageIndex(page - 1)}
              />
            </Group>
          </Group>
        </Box>
      ) : null}
    </SurfaceCard>
  );
}
