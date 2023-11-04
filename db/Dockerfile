# 使用官方的postgres镜像作为基础镜像
FROM postgres:latest

# 设置环境变量
ENV POSTGRES_USER miniprogram
ENV POSTGRES_PASSWORD passwd
ENV POSTGRES_DB miniprogram
ADD ./init.sql /docker-entrypoint-initdb.d/