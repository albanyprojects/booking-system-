# booking-system-
a booking system, can book holidays, calculates term times and bank holidays for england and wales, calculates holiday crossovers, so if theres boxing day on a sunday, the bank holiday crosses over to, well, it would be a tuesday, because boxing day sunday means christmas saturday which also needs substituting to the monday. also accounts for leave


the system also contains code for an eventual calendar. this calendar is central to the rule requests in booking.rs, as it is meant to ensure that users cannot boof days of on A) bank holidays, B) Weekends, C) when too many other people have booked the time off work. the calendars functions as of now remain unused, although there should be no errors compiling since main.rs ensures unused code doesnt provide warnings. 


this specifically is an older and more outdated and possibly not even working version of the file, 

this is a backend logic system and it isnt designed to run on github.        

to fix this: change the business logic to remove hardcoded data

code to take information from database of choice, you do need a database server

run endpoints through with dummy data to make sure the logic works

develop a frontend GUI to interact with the system., 

