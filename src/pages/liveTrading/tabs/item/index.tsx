import { useTranslateEnums, useTranslatePages } from "@hooks/index";
import { useLiveScraperContext, useStockContextContext } from "@contexts/index";
import { sortArray, paginate, getCssVariable, GetSubTypeDisplay } from "@utils/index";
import { useEffect, useState } from "react";
import { DataTable, DataTableSortStatus } from "mantine-datatable";
import { CreateStockItem, StockItem, StockStatus, UpdateStockItem, SellStockItem } from "@api/types";
import { ColorInfo, CreateStockItemForm, SearchField, StatsWithSegments, ButtonInterval, TextTranslate, StockItemInfo } from "@components";
import { ActionIcon, Box, Grid, Group, NumberFormatter, Text, Tooltip } from "@mantine/core";
import { useMutation } from "@tanstack/react-query";
import api from "@api/index";
import { notifications } from "@mantine/notifications";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faEdit, faEye, faEyeSlash, faHammer, faInfo, faPen, faTrashCan } from "@fortawesome/free-solid-svg-icons";
import { modals } from "@mantine/modals";
interface StockItemPanelProps {
}
export const StockItemPanel = ({ }: StockItemPanelProps) => {
    // States Context 
    const { items } = useStockContextContext();
    const { is_running } = useLiveScraperContext();

    // States For DataGrid
    const [page, setPage] = useState(1);
    const pageSizes = [5, 10, 15, 20, 25, 30, 50, 100];
    const [pageSize, setPageSize] = useState(pageSizes[4]);
    const [rows, setRows] = useState<StockItem[]>([]);
    const [totalRecords, setTotalRecords] = useState<number>(0);
    const [sortStatus, setSortStatus] = useState<DataTableSortStatus<StockItem>>({ columnAccessor: 'name', direction: 'desc' });

    const [query, setQuery] = useState<string>("");
    const [filterStatus, setFilterStatus] = useState<StockStatus | undefined>(undefined);
    const [statusCount, setStatusCount] = useState<{ [key: string]: number }>({}); // Count of each status


    const [segments, setSegments] = useState<{ label: string, count: number, part: number, color: string }[]>([]);

    // Translate general
    const useTranslate = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslatePages(`liveTrading.${key}`, { ...context }, i18Key)
    const useTranslateSegments = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslate(`segments.${key}`, { ...context }, i18Key)
    const useTranslateTabItem = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslate(`tabs.item.${key}`, { ...context }, i18Key)
    const useTranslateStockStatus = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateEnums(`stock_status.${key}`, { ...context }, i18Key)
    const useTranslateDataGridColumns = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateTabItem(`datatable.columns.${key}`, { ...context }, i18Key)
    const useTranslateDataGridBaseColumns = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslate(`datatable.columns.${key}`, { ...context }, i18Key)
    const useTranslateErrors = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateTabItem(`errors.${key}`, { ...context }, i18Key)
    const useTranslateSuccess = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateTabItem(`success.${key}`, { ...context }, i18Key)
    const useTranslatePrompt = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslate(`prompts.${key}`, { ...context }, i18Key)
    const useTranslateNotifications = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslate(`notifications.${key}`, { ...context }, i18Key)

    // Update Database Rows
    useEffect(() => {
        if (!items)
            return;

        let rivensFilter = items;

        setStatusCount(Object.values(StockStatus).reduce((acc, status) => {
            acc[status] = rivensFilter.filter((item) => item.status === status).length;
            return acc;
        }, {} as { [key: string]: number }));

        if (query !== "")
            rivensFilter = rivensFilter.filter((item) => item.item_name.toLowerCase().includes(query.toLowerCase()));

        if (filterStatus)
            rivensFilter = rivensFilter.filter((item) => item.status === filterStatus);

        setTotalRecords(rivensFilter.length);
        rivensFilter = sortArray([{
            field: sortStatus.columnAccessor,
            direction: sortStatus.direction
        }], rivensFilter);



        rivensFilter = paginate(rivensFilter, page, pageSize);
        setRows(rivensFilter);


    }, [items, query, pageSize, page, sortStatus, filterStatus])

    // Calculate Stats
    useEffect(() => {
        if (!items) return;
        const totalPurchasePrice = items.reduce((a, b) => a + (b.bought || 0) * b.owned, 0);
        const totalListedPrice = items.reduce((a, b) => a + (b.list_price || 0) * b.owned, 0);
        const totalProfit = totalListedPrice > 0 ? (totalListedPrice - totalPurchasePrice) : 0;

        // Calculate the total count
        const totalCount = totalPurchasePrice + totalListedPrice + totalProfit;

        // Calculate the percentage of each count relative to the total count
        const boughtPercentage = (totalPurchasePrice / totalCount) * 100;
        const listedPercentage = (totalListedPrice / totalCount) * 100;
        const profitPercentage = (totalProfit / totalCount) * 100;
        setSegments([
            { label: useTranslateSegments("bought"), count: totalPurchasePrice, part: boughtPercentage, color: getCssVariable("--negative-value") },
            { label: useTranslateSegments("listed"), count: totalListedPrice, part: listedPercentage, color: getCssVariable("--positive-value") },
            { label: useTranslateSegments("profit"), count: totalProfit, part: profitPercentage, color: getCssVariable("--profit-value") },
        ]);
    }, [items])

    // Mutations
    const createStockMutation = useMutation({
        mutationFn: (data: CreateStockItem) => api.stock.item.create(data),
        onSuccess: async (u) => {
            notifications.show({ title: useTranslateSuccess("create_stock.title"), message: useTranslateSuccess("create_stock.message", { name: u.item_name }), color: "green.7" });
        },
        onError: (e) => {
            console.error(e);
            notifications.show({ title: useTranslateErrors("create_stock.title"), message: useTranslateErrors("create_stock.message"), color: "red.7" })
        }
    })
    const updateStockMutation = useMutation({
        mutationFn: (data: UpdateStockItem) => api.stock.item.update(data),
        onSuccess: async (u) => {
            notifications.show({ title: useTranslateSuccess("update_stock.title"), message: useTranslateSuccess("update_stock.message", { name: u.item_name }), color: "green.7" });
        },
        onError: (e) => {
            console.error(e);
            notifications.show({ title: useTranslateErrors("update_stock.title"), message: useTranslateErrors("update_stock.message"), color: "red.7" })
        }
    })
    const sellStockMutation = useMutation({
        mutationFn: (data: SellStockItem) => api.stock.item.sell(data),
        onSuccess: async (u) => {
            notifications.show({ title: useTranslateSuccess("sell_stock.title"), message: useTranslateSuccess("sell_stock.message", { name: u.item_name }), color: "green.7" });
        },
        onError: (e) => {
            console.error(e);
            notifications.show({ title: useTranslateErrors("sell_stock.title"), message: useTranslateErrors("sell_stock.message"), color: "red.7" })
        }
    })
    const deleteStockMutation = useMutation({
        mutationFn: (id: number) => api.stock.item.delete(id),
        onSuccess: async () => {
            notifications.show({ title: useTranslateSuccess("delete_stock.title"), message: useTranslateSuccess("delete_stock.message"), color: "green.7" });
        },
        onError: (e) => {
            console.error(e);
            notifications.show({ title: useTranslateErrors("delete_stock.title"), message: useTranslateErrors("delete_stock.message"), color: "red.7" })
        }
    })

    // Modal's
    const OpenMinimumPriceModal = (id: number, minimum_price: number) => {
        modals.openContextModal({
            modal: 'prompt',
            title: useTranslatePrompt('minimum_price.title'),
            innerProps: {
                fields: [
                    {
                        name: 'minimum_price',
                        label: useTranslatePrompt('minimum_price.fields.minimum_price.label'),
                        attributes: {
                            min: 0,
                            description: useTranslatePrompt('minimum_price.fields.minimum_price.description')
                        },
                        value: minimum_price,
                        type: 'number',
                    },
                ],
                onConfirm: async (data: { minimum_price: number }) => {
                    if (!id) return;
                    const { minimum_price } = data;
                    await updateStockMutation.mutateAsync({ id, minimum_price })
                },
                onCancel: (id: string) => modals.close(id),
            },
        })
    }

    const OpenSellModal = (id: number) => {
        modals.openContextModal({
            modal: 'prompt',
            title: useTranslatePrompt('sell.title'),
            innerProps: {
                fields: [
                    {
                        name: 'sell',
                        label: useTranslatePrompt('sell.fields.sell.label'),
                        attributes: {
                            min: 0,
                        },
                        value: 0,
                        type: 'number',
                    },
                ],
                onConfirm: async (data: { sell: number }) => {
                    if (!id) return;
                    const { sell } = data;
                    await sellStockMutation.mutateAsync({ id, price: sell, quantity: 1 })
                },
                onCancel: (id: string) => modals.close(id),
            },
        })
    }

    const OpenInfoModal = (item: StockItem) => {
        modals.open({
            size: "100%",
            title: item.item_name,
            children: (<StockItemInfo value={item} />),

        })
    }

    return (
        <Box>
            <Grid>
                <Grid.Col span={8}>
                    <CreateStockItemForm disabled={createStockMutation.isPending || updateStockMutation.isPending || sellStockMutation.isPending || deleteStockMutation.isPending} onSubmit={async (item) => {
                        createStockMutation.mutate(item);
                    }} />
                    <Group gap={"md"} mt={"md"}>
                        {Object.values(StockStatus).map((status) => (
                            <ColorInfo active={status == filterStatus} key={status} onClick={() => {
                                setFilterStatus(s => s === status ? undefined : status);
                            }} infoProps={{
                                "data-color-mode": "bg",
                                "data-stock-status": status,
                            }} text={useTranslateStockStatus(`${status}`) + `${statusCount[status] == 0 ? "" : ` (${statusCount[status]})`}`} tooltip={useTranslateStockStatus(`details.${status}`)} />
                        ))}
                    </Group>
                </Grid.Col>
                <Grid.Col span={4}>
                    <StatsWithSegments segments={segments} />
                </Grid.Col>
            </Grid>
            <SearchField value={query} onChange={(text) => setQuery(text)} />
            <DataTable
                height={`calc(100vh - ${!is_running ? "400px" : "420px"})`}
                mt={"md"}
                records={rows}
                totalRecords={totalRecords}
                customRowAttributes={(record) => {
                    return {
                        "data-color-mode": "box-shadow",
                        "data-stock-status": record.status,
                    }
                }}
                withTableBorder
                fetching={createStockMutation.isPending || updateStockMutation.isPending || sellStockMutation.isPending || deleteStockMutation.isPending}
                withColumnBorders
                page={page}
                recordsPerPage={pageSize}
                idAccessor={"id"}
                onPageChange={(p) => setPage(p)}
                recordsPerPageOptions={pageSizes}
                onRecordsPerPageChange={setPageSize}
                sortStatus={sortStatus}
                onSortStatusChange={setSortStatus}
                onCellClick={({ record, column }) => {
                    switch (column.accessor) {
                        case "item_name":
                            navigator.clipboard.writeText(record.item_name);
                            notifications.show({ title: useTranslateNotifications("copied.title"), message: record.item_name, color: "green.7" });
                            break;
                    }
                }}
                // define columns
                columns={[
                    {
                        accessor: 'item_name',
                        title: useTranslateDataGridBaseColumns('name.title'),
                        sortable: true,
                        render: ({ item_name, sub_type }) => (
                            <TextTranslate color="gray.4" i18nKey={useTranslateDataGridBaseColumns("name.value", undefined, true)} values={{
                                name: item_name,
                                sub_type: GetSubTypeDisplay(sub_type)
                            }} />
                        ),
                    },
                    {
                        accessor: 'bought',
                        title: useTranslateDataGridBaseColumns('bought'),
                        sortable: true,
                        render: ({ bought }) => <NumberFormatter thousandSeparator="." decimalSeparator="," value={bought} />,
                    },
                    {
                        accessor: 'minimum_price',
                        width: 310,
                        title: useTranslateDataGridBaseColumns('minimum_price.title'),
                        render: ({ id, minimum_price }) => (
                            <Group gap={"sm"} justify="space-between">
                                <Text>{minimum_price || "N/A"}</Text>
                                <Group gap={"xs"}>
                                    <ButtonInterval color="red.7" intervals={[5, 10]} prefix="-" OnClick={async (int) => {
                                        if (!id) return;
                                        minimum_price = minimum_price || 0;
                                        if (minimum_price - int < 0) return;
                                        await updateStockMutation.mutateAsync({ id, minimum_price: minimum_price - int });
                                    }} />
                                    <ButtonInterval color="green.7" intervals={[5, 10]} prefix="+" OnClick={async (int) => {
                                        if (!id) return;
                                        minimum_price = minimum_price || 0;
                                        await updateStockMutation.mutateAsync({ id, minimum_price: minimum_price + int });
                                    }} />
                                    <Tooltip label={useTranslateDataGridBaseColumns('minimum_price.btn.edit.tooltip')} position="top">
                                        <ActionIcon size={"sm"} color={"blue.7"} variant="filled" onClick={async (e) => {
                                            e.stopPropagation();
                                            if (!id) return;
                                            OpenMinimumPriceModal(id, minimum_price || 0);
                                        }} >
                                            <FontAwesomeIcon size="xs" icon={faEdit} />
                                        </ActionIcon>
                                    </Tooltip>
                                </Group>
                            </Group>
                        ),
                    },
                    {
                        accessor: 'list_price',
                        title: useTranslateDataGridBaseColumns('list_price'),
                    },
                    {
                        accessor: 'owned',
                        title: useTranslateDataGridColumns('owned'),
                    },
                    {
                        accessor: 'actions',
                        title: useTranslateDataGridBaseColumns('actions.title'),
                        width: 180,
                        render: (row) => (
                            <Group gap={"sm"} justify="flex-end">
                                <Tooltip label={useTranslateDataGridBaseColumns('actions.buttons.sell_manual.tooltip')} position="top">
                                    <ActionIcon size={"sm"} color={"green.7"} variant="filled" onClick={async (e) => {
                                        e.stopPropagation();
                                        OpenSellModal(row.id);
                                    }} >
                                        <FontAwesomeIcon size="xs" icon={faPen} />
                                    </ActionIcon>
                                </Tooltip>
                                <Tooltip label={useTranslateDataGridBaseColumns('actions.buttons.sell_auto.tooltip')} position="top">
                                    <ActionIcon disabled={!row.list_price} size={"sm"} color={"blue.7"} variant="filled" onClick={async (e) => {
                                        e.stopPropagation();
                                        if (!row.id || !row.list_price) return;
                                        await sellStockMutation.mutateAsync({ id: row.id, price: row.list_price, quantity: 1 });
                                    }} >
                                        <FontAwesomeIcon size="xs" icon={faHammer} />
                                    </ActionIcon>
                                </Tooltip>
                                <Tooltip label={useTranslateDataGridBaseColumns(`actions.buttons.hide.${row.is_hidden ? "disabled_tooltip" : "enabled_tooltip"}`)} position="top">
                                    <ActionIcon size={"sm"} color={`${row.is_hidden ? "red.7" : "green.7"}`} variant="filled" onClick={async (e) => {
                                        e.stopPropagation();
                                        await updateStockMutation.mutateAsync({ id: row.id, is_hidden: !row.is_hidden });
                                    }} >
                                        <FontAwesomeIcon size="xs" icon={row.is_hidden ? faEyeSlash : faEye} />
                                    </ActionIcon>
                                </Tooltip>
                                <Tooltip label={useTranslateDataGridBaseColumns('actions.buttons.info.tooltip')} position="top">
                                    <ActionIcon size={"sm"} color={"blue.7"} variant="filled" onClick={async (e) => {
                                        e.stopPropagation();
                                        OpenInfoModal(row);
                                    }} >
                                        <FontAwesomeIcon size="xs" icon={faInfo} />
                                    </ActionIcon>
                                </Tooltip>
                                <Tooltip label={useTranslateDataGridBaseColumns('actions.buttons.delete.tooltip')} position="top">
                                    <ActionIcon size={"sm"} color={"red.7"} variant="filled" onClick={async (e) => {
                                        e.stopPropagation();
                                        await deleteStockMutation.mutateAsync(row.id);
                                    }} >
                                        <FontAwesomeIcon size="xs" icon={faTrashCan} />
                                    </ActionIcon>
                                </Tooltip>
                            </Group>
                        ),
                    },
                ]} />
        </Box>
    );
};