define sales : read-csv "sales.csv"

define summary
    -> sales
        with-column $revenue {$quantity * $price}
        group-by $category
            alias (sum $revenue) "total_revenue"
        sort $total_revenue :desc

print summary
write-csv summary "summary.csv"
