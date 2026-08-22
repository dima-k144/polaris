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

(histogram lf
  :x "sales"
  :title "Sales Distribution"
  :x-title "Sales Amount ($)"
  :y-title "Frequency")

(histogram lf
  :x "profit"
  :group "region"
  :opacity 0.6
  :title "Profit Distribution by Region"
  :x-title "Profit ($)"
  :y-title "Count")

(box-plot lf
  :labels "category"
  :values "profit"
  :title "Profit Spread across Categories"
  :x-title "Category"
  :y-title "Profit ($)")

(time-series-plot lf
  :x "date"
  :y "sales"
  :title "Daily Sales Over Time"
  :x-title "Date"
  :y-title "Sales ($)")

(print lf)
