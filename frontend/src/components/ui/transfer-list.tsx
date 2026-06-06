"use client";

import { Checkbox, Group, ScrollArea, Stack, Text } from "@mantine/core";
import { IconArrowLeft, IconArrowRight } from "@tabler/icons-react";
import { AppButton } from "./button";
import { SearchInput } from "./search-input";
import { SurfaceCard } from "./surface-card";
import { useMemo, useState } from "react";

export type TransferItem = {
  description?: string;
  disabled?: boolean;
  label: string;
  value: string;
};

type TransferListProps = {
  availableLabel?: string;
  items: TransferItem[];
  onChange: (values: string[]) => void;
  selectedLabel?: string;
  value: string[];
};

export const TransferList = ({
  availableLabel = "Available",
  items,
  onChange,
  selectedLabel = "Selected",
  value,
}: TransferListProps) => {
  const [availableQuery, setAvailableQuery] = useState("");
  const [selectedQuery, setSelectedQuery] = useState("");
  const [checkedAvailable, setCheckedAvailable] = useState<string[]>([]);
  const [checkedSelected, setCheckedSelected] = useState<string[]>([]);
  const selectedSet = useMemo(() => new Set(value), [value]);

  const availableItems = useMemo(
    () =>
      items.filter(
        (item) =>
          !selectedSet.has(item.value) &&
          item.label.toLowerCase().includes(availableQuery.toLowerCase()),
      ),
    [availableQuery, items, selectedSet],
  );

  const selectedItems = useMemo(
    () =>
      items.filter(
        (item) =>
          selectedSet.has(item.value) &&
          item.label.toLowerCase().includes(selectedQuery.toLowerCase()),
      ),
    [items, selectedQuery, selectedSet],
  );

  const moveToSelected = () => {
    onChange(Array.from(new Set(value.concat(checkedAvailable))));
    setCheckedAvailable([]);
  };

  const moveToAvailable = () => {
    onChange(value.filter((item) => !checkedSelected.includes(item)));
    setCheckedSelected([]);
  };

  const renderList = (
    label: string,
    listItems: TransferItem[],
    checked: string[],
    setChecked: (values: string[]) => void,
    query: string,
    setQuery: (value: string) => void,
  ) => (
    <SurfaceCard title={label}>
      <Stack spacing="xs" p="sm">
        <SearchInput value={query} onChange={setQuery} placeholder={`Search ${label.toLowerCase()}`} />
        <ScrollArea h={280}>
          <Stack spacing={4}>
            {listItems.map((item) => (
              <Checkbox
                key={item.value}
                color="cyan"
                disabled={item.disabled}
                checked={checked.includes(item.value)}
                label={
                  <div>
                    <Text size="sm">{item.label}</Text>
                    {item.description ? (
                      <Text size="xs" color="dimmed">
                        {item.description}
                      </Text>
                    ) : null}
                  </div>
                }
                onChange={(event) => {
                  if (event.currentTarget.checked) setChecked([...checked, item.value]);
                  else setChecked(checked.filter((value) => value !== item.value));
                }}
              />
            ))}
          </Stack>
        </ScrollArea>
      </Stack>
    </SurfaceCard>
  );

  return (
    <Group grow align="stretch">
      {renderList(
        availableLabel,
        availableItems,
        checkedAvailable,
        setCheckedAvailable,
        availableQuery,
        setAvailableQuery,
      )}
      <Stack justify="center" align="center" spacing="xs" sx={{ flexGrow: 0 }}>
        <AppButton
          icon={<IconArrowRight size={15} />}
          onClick={moveToSelected}
          disabled={!checkedAvailable.length}
        >
          Add
        </AppButton>
        <AppButton
          appVariant="secondary"
          icon={<IconArrowLeft size={15} />}
          onClick={moveToAvailable}
          disabled={!checkedSelected.length}
        >
          Remove
        </AppButton>
      </Stack>
      {renderList(
        selectedLabel,
        selectedItems,
        checkedSelected,
        setCheckedSelected,
        selectedQuery,
        setSelectedQuery,
      )}
    </Group>
  );
};
