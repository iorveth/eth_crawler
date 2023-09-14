# eth_crawler

## Set up
1. Create `.env` file following `.example_env`
2. Install mysql server `sudo apt install mysql-server`
3. Ensure that the server is running using the systemctl start command: `sudo systemctl start mysql.service`
4. `sudo mysql`
5. `ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY 'root';`
6. `CREATE DATABASE transaction_data;`
7.  After making this changes, exit the MySQL prompt:
`exit`

## Run
1. `cargo run`
2. go to `http://127.0.0.1:8000/`![Screenshot from 2023-09-14 17-24-14](https://github.com/iorveth/eth_crawler/assets/18070359/4b42b67a-d738-4775-8e98-e775fb32a024)
3. check parsed transactions ![Screenshot from 2023-09-14 17-27-49](https://github.com/iorveth/eth_crawler/assets/18070359/17487f33-5889-4b74-9803-b9cda83b1f8a)

