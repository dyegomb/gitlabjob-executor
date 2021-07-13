from getpass import getpass
from os import environ
from email.message import EmailMessage
import smtplib

class Jobmail():
    def __init__(self, server="", auth=False, 
                login="", password=""):
        self.server = server or environ.get("SMTP_SERVER")

        self.auth = auth
        if auth or environ.get("SMTP_USER"):
            self._login = login or environ["SMTP_USER"]
            self.__passw = password or environ.get("SMTP_PASS") \
                    or getpass("Informe a senha:")
            self.auth = True
        
        self.msg = EmailMessage()
        self.msg["Body"] = ""
        self.corpo = ""
        self.destino = list()

    def assunto(self, assunto:str):
        self.msg["Subject"] = assunto

    def destino_add(self, dest:str):
        if type(dest) is not str: return False
        if not dest: return False
        self.destino.append(dest)
        return True

    def remetente(self, remetente:str):
        self.msg["From"] = remetente

    def msg_append(self, conteudo:str):
        self.corpo += "\n"+conteudo
        self.msg.set_content(self.corpo)

    def send(self):
        self.msg["To"] = ",".join(self.destino)
        with smtplib.SMTP(self.server) as server:
            if self.auth:
                server.login(self._login, self.__passw)
            
            server.send_message(self.msg)
