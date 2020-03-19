FROM node:lts-alpine

WORKDIR /ui
RUN apk --no-cache add ca-certificates
EXPOSE 8080
CMD npm install && npm run serve
