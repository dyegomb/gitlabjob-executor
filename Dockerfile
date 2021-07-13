FROM python3

USER root

RUN  pip3 install requests

COPY *.py /opt/

WORKDIR /opt
