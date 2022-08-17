FROM node:16

WORKDIR /helibot

COPY package*.json ./

RUN npm i
RUN npm i -g typescript

COPY ./ ./

RUN tsc

EXPOSE 2832

CMD ["node", "out/main.js"]
