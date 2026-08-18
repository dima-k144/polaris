(define df (read-csv "data1.csv"))

(print (head df))
(print (tail df))
(print (slice df 1 2))
(print (schema df))
(print (shape df))

(print (-> df
    (filter 
        (= $"age" (lit 25))
    )
))

(print (-> df
    (filter 
        (= $"name" (lit "Bob"))
    )
))

