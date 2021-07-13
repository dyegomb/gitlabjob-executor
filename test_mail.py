from jobmail import Jobmail
from os import environ
import unittest


class Testes(unittest.TestCase):
    @classmethod
    def setUpClass(self):
        self.jobmail = Jobmail()
        
    def test_headers(self):
        assunto = environ.get("SMTP_SUBJECT") or "Teste assunto"
        self.jobmail.assunto(assunto)
        self.assertEqual(self.jobmail.msg["Subject"], assunto, "Erro ao\
 definir assunto")

        destino = environ.get("SMTP_TO") or "teste@test.tst"
        self.jobmail.destino.append(destino)
        self.assertIn(destino, self.jobmail.destino, "Erro ao \
definir destino")

        remetente = environ.get("SMTP_FROM") or "teste1@teste.tst"
        self.jobmail.remetente(remetente)
        self.assertEqual(self.jobmail.msg["From"], remetente, "Erro ao\
 definir remetente")

    def test_body(self):
        msg = "Teste conteúdo"
        self.jobmail.msg_append(msg)
        self.assertIn(msg, self.jobmail.msg.get_payload())

        msg2 = "Teste 2"
        self.jobmail.msg_append(msg2)
        self.assertIn(msg2, self.jobmail.msg.get_payload())

    def test_appendto(self):
        destino = environ.get("SMTP_TO") or "teste@test.tst"
        self.jobmail.destino_add(destino)
        self.assertIn(destino, self.jobmail.destino, "Erro ao \
definir múltiplos destino")

    def test_send(self):
        self.jobmail.send()

if __name__ == "__main__":
    unittest.main()
