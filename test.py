from gitlabjob import GitlabJob
from time import sleep
from os import environ
import unittest

class Testes(unittest.TestCase):
    @classmethod
    def setUpClass(self, proj_id="306"):
        self.prjid = str(environ.get("PROJECT_ID", proj_id))
        self.gitjob = GitlabJob(project_id=self.prjid)

        self.form_trigger = {
            "token": environ.get("TESTE_TOKENTRIG", 
                    "ceacca74cb3fdeed161d8552870c63"),
            "ref": "master",
            "variables[trigger_email]": environ.get("SMTP_TO") or "teste@teste.tst",
        }

        self.url_trigger = f"api/v4/projects/{proj_id}/trigger/pipeline"

    def teste1(self):
        self.assertIsInstance(self.gitjob.get_jobs(filtro={"status":"manual"}), list)

    def teste2_filter(self):
        self.assertEqual(self.gitjob._filter([1,2,3], 0), None) 
        try:
            self.gitjob._filter(1, 0)
            self.assertTrue(False)
        except:
            self.assertTrue(True)

        # Resposta de informação de variáveis de pipeline
        texto_json = [{"variable_type":"env_var","key":"ref_source","value":"homolog"},{"variable_type":"env_var","key":"trigger_email","value":"teste@teste.tst"}]

        for item in texto_json:
            mail = self.gitjob._filter(item, "key", "trigger_email")
            if mail: break

        self.assertEqual(mail, texto_json[1])

    def teste3_play(self):

        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)

        sleep(5)

        jobs_list = self.gitjob.get_jobs(filtro={"status":"manual"})
        if not jobs_list:
            raise Exception("Não há jobs manuais para execução")

        for job in jobs_list:
            self.assertEqual(self.gitjob.play_job(job), 200, 
                f"Não houve sucesso ao iniciar job: {job}")

    def teste4_playall(self):

        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)

        sleep(5)

        status_dict = self.gitjob.play_all()
        for job in status_dict.items():
            self.assertEqual(job[1], 200, f"não foi possível iniciar o job {job[0]} corretamente")

    def teste5_jobinfo(self):
        jobs_list = self.gitjob.get_jobs(filtro=None)

        infos = self.gitjob.get_jobinfo(jobs_list[0])
        self.assertGreater(len(infos),1)
        self.assertIsInstance(infos, dict)

    def teste6_grupo(self):
        grp_id = environ.get("GROUP_ID") or "10"
        projs_ids = self.gitjob.group_projs(grp_id)

        self.assertIsInstance(projs_ids, list)



    @classmethod
    def tearDownClass(self):
        """Adicona novo job para posterior execução"""
        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)



if __name__ == "__main__":
    unittest.main()