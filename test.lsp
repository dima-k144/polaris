(define df (read-csv "data1.csv"))

(print (head df))
(print (tail df))
(print (slice df 1 2))
(print (schema df))
(print (shape df))

(print (-> df
    (filter 
        (= $"age" 25)
    )
))

(print (-> df
    (filter 
        (= $"name" "Bob")
    )
))

(with-column df $"is_adult" (>= $"age" 18))
(with-column df $"is_old" (>= $"age" 30))

(print (sort df $"age"))
(print (sort df $"age" :desc))
