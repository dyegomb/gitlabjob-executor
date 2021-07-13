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
            "token": environ["TESTE_TOKENTRIG"],
            "ref": "master",
            "variables[trigger_email]": environ.get("SMTP_TO") or "teste@test.tst",
            "variables[source_id]": self.prjid,
            "variables[ref_source]": "master",
            "variables[PROD_TAG]": "PROD-1.0"
        }

        self.url_trigger = f"api/v4/projects/{proj_id}/trigger/pipeline"

    def teste1(self):
        """Função get_jobs"""
        self.assertIsInstance(self.gitjob.get_jobs(filtro={"status":"manual"}), list)

    def teste2_filter(self):
        """Testa filtro"""
        self.assertEqual(self.gitjob._filter([1,2,3], 0), None) 
        try:
            self.gitjob._filter(1, 0)
            self.assertTrue(False)
        except:
            self.assertTrue(True)

        # Resposta de informação de variáveis de pipeline
        texto_json = [{"variable_type":"env_var","key":"ref_source",
        "value":"homolog"},{"variable_type":"env_var","key":"trigger_email",
        "value":"teste@teste.tst"}]

        for item in texto_json:
            mail = self.gitjob._filter(item, "key", "trigger_email")
            if mail: break

        self.assertEqual(mail, texto_json[1])

    def teste3_play(self):
        """Execução de job"""

        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)

        sleep(2)

        jobs_list = self.gitjob.get_jobs(filtro={"status":"manual"})
        if not jobs_list:
            raise Exception("Não há jobs manuais para execução")

        for job in jobs_list:
            self.assertEqual(self.gitjob.play_job(job), 200, 
                f"Não houve sucesso ao iniciar job: {job}")

    def teste4_playall(self):
        """Função play_all"""

        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)

        sleep(5)

        status_dict = self.gitjob.play_all()
        for job in status_dict.items():
            self.assertEqual(job[1], 200, 
                f"não foi possível iniciar o job {job[0]} corretamente")

    def teste5_jobinfo(self):
        """Capturar informações de jobs"""
        jobs_list = self.gitjob.get_jobs(filtro={"status":"success"})

        infos = self.gitjob.get_jobinfo(jobs_list[0])
        self.assertGreater(len(infos),1)
        self.assertIsInstance(infos, dict)

        self.assertEqual(infos.get("source_id"), self.gitjob.project_id,
                         infos)

        self.assertEqual("PROD-1.0", infos.get("versao_tag"))

    def teste6_grupo(self):
        """Teste de listagem de projetos em grupos do Gitlab"""
        grp_id = environ.get("GROUP_ID")
        projs_ids = self.gitjob.group_projs(grp_id)

        self.assertIsInstance(projs_ids, list)

    def teste7_deletepipelines(self):
        """Exclui pipelines duplicadas"""

        # Adiciona múltiplas pipelines
        for _ in range(2):
            sleep(3)
            self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)

        pipes_list = self.gitjob.get_pipelines()

        self.assertGreaterEqual(len(pipes_list), 2)

        for pipe in pipes_list[:-1]:
            status_code = self.gitjob.delete_pipeline(pipe)
            self.assertEqual(status_code, 204)

        sleep(5)
        pipes_list = self.gitjob.get_pipelines()
        self.assertEqual(len(pipes_list), 1, f"Deveria existir apenas uma pipeline: {pipes_list}")

    def teste8_tags(self):
        """Teste para TAGs GIT"""
        tags = self.gitjob.get_tags()

        self.assertIsInstance(tags, list, "Não houve retorno de lista de tags")
        self.assertGreaterEqual(len(tags), 1, "Tags não encontradas")

        tags = self.gitjob.get_tags(filtro="^PROD")

        self.assertIsInstance(tags, list, "Não houve retorno de lista de tags")
        self.assertGreaterEqual(len(tags), 1, "TAGs \"PROD\" não encontradas")

    @classmethod
    def tearDownClass(self):
        """Adicona novo job para posterior execução"""
        self.gitjob._req(self.url_trigger, "POST", data=self.form_trigger)



if __name__ == "__main__":
    unittest.main()
