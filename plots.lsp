(define lf 
    (-> (read-csv "data6.csv")
        (with-column $"date" (to-date $"date" "%Y-%m-%d"))
        (sort $"date")))

(scatter-plot lf
    :title "plot 1"
    :x "sales"
    :y "profit"
)

(define sales-by-cat
  (group-by lf $"category" (alias (sum $"sales") "total_sales")))

(bar-plot sales-by-cat
  :labels "category"
  :values "total_sales"
  :title "Total Sales by Category"
  :x-title "Category"
  :y-title "Total Sales ($)")

(pie-plot lf
  :labels "category"
  :title "Distribution by Category")

(print lf)
