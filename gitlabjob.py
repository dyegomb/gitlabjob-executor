import requests
from os import environ


class GitlabJob(object):
    def __init__(self, token:str="", project_id:str="", base_url:str=""):
        self.pvtoken = token or environ["PRIVATE_TOKEN"]
        self.project_id = project_id or environ.get("PROJECTS_ID")
        self.base_url = base_url or environ["BASE_URL"]

        self.jobs_list = list()
    
    def _req(self, uri="/", method="GET", data="", 
        headers:dict={}) -> requests.Response:

        sessao = requests.Session()

        sessao.headers['PRIVATE-TOKEN'] = self.pvtoken
        sessao.headers['Accept'] = 'application/json'
        for head in headers.items():
            sessao.headers[head[0]] = head[1]

        joinner = "" if self.base_url.endswith("/") else "/"
        url = joinner.join([self.base_url, uri])

        try:
            resp = sessao.request(method, url, data=data)
        finally:
            sessao.close()

        return resp

    @staticmethod
    def _filter(item:iter, key:str, value=""):
        assert type(item) in [list, tuple, dict, set], "Tipo de objeto inválido"
        if isinstance(item, dict):
            if item.get(key) == value:
                return item
            else:
                return None

        if key in item:
            return item 
        else:
            return None

    def get_jobs(self, proj_id="", filtro=dict()) -> list: 
        """
        Trás lista de JOBs de determinado projeto.

        .get_jobs(filtro={"status": "manual"}) -> 
        ['jobid', 'jobid']
        """
        projid = proj_id or self.project_id
        uri = f'/api/v4/projects/{projid}/jobs?pagination=keyset&per_page=100&order_by=id&sort=asc'
        if filtro and "status" in filtro.keys(): 
            scope = filtro["status"]
            uri += f"&scope={scope}"
        resp = self._req(uri)

        rjson = resp.json()
        for pagina in range(1, int(resp.headers.get("x-total-pages"))+1):
            if pagina == 1: continue
            uri_pg = uri+f"&page={pagina}"
            pg_resp = self._req(uri_pg)
            rjson.extend(pg_resp.json())

        jobs_list = list()
        for job in rjson:
            if filtro:
                filtrar = [list(filtro.keys())[0], list(filtro.values())[0]]
                item = self._filter(job, *filtrar)
            else:
                item = job

            if item:
                jobs_list.append(item.get("id"))
        
        return jobs_list

    def play_job(self, jobid:str, proj_id=""):
        prj_id = proj_id or self.project_id
        uri = f'/api/v4/projects/{prj_id}/jobs/{jobid}/play'
        resultado = self._req(uri, "POST")
        return resultado.status_code

    def play_all(self, filtro={"status":"manual"}, proj_id="") -> dict:
        prj_id = proj_id or self.project_id
        retorno = dict()
        self.jobs_list = self.get_jobs(filtro=filtro, proj_id=prj_id)

        for job in self.jobs_list:
            status = self.play_job(job)
            retorno[job] = status

        return retorno

    def get_jobinfo(self, jobid:str, proj_id="") -> dict:
        prj_id = proj_id or self.project_id
        uri = f'/api/v4/projects/{prj_id}/jobs/{jobid}'
        resultado = self._req(uri)
        try:
            jobs_json = resultado.json()
        except Exception:
            return {"erro": "não foi possível capturar informações do job",
                    "codigo": resultado.status_code }


        pipid = jobs_json.get("pipeline").get("id")
        uri = f'/api/v4/projects/{prj_id}/pipelines/{pipid}/variables'
        resultado = self._req(uri)

        try:
            pipe_json = resultado.json()
        except Exception:
            return {"erro": "não foi possível capturar informações do pipeline",
                    "codigo": resultado.status_code }
        
        user_mail = ""
        prod_tag = ""
        ref_source = ""
        for item in pipe_json:
            try:
                user_mail = self._filter(item, "key", "trigger_email"
                                        ).get("value") or user_mail
                prod_tag = self._filter(item, "key", "PROD_TAG"
                                        ).get("value") or prod_tag
                ref_source = self._filter(item, "key", "ref_source"
                                        ).get("value") or ref_source
            except AttributeError:
                continue

        uri = f'/api/v4/projects/{prj_id}'
        resultado = self._req(uri)
        try:
            proj_json = resultado.json()
        except Exception:
            return {"erro": "não foi possível capturar informações de projeto",
                    "codigo": resultado.status_code }

        infos = {
            'jobid': jobs_json.get("id"),
            'job_url': jobs_json.get("web_url"),
            'nome_projeto': proj_json.get("name"),
            "pipelineid": pipid,
            "user_mail": user_mail, 
            "branch": ref_source or "não informada",
            "versao_tag": prod_tag or "não informada",
        }

        return infos

    def group_projs(self, groupid="") -> list:
        uri = f"/api/v4/groups/{groupid}/projects?pagination=keyset&per_page=100&order_by=id&sort=asc"

        grp_projs = self._req(uri).json()

        projs_ids = list()
        for proj in grp_projs:
            projs_ids.append(proj.get("id"))

        return projs_ids


if __name__ == "__main__":
    from argparse import ArgumentParser

    parser = ArgumentParser()

    parser.add_argument("-b", "--url-base", help="URL base do Gitlab", 
    metavar="https://gitlab.com/user")
    parser.add_argument("-t", "--private-token", help="Private token com\
 permissões necessárias", metavar="1234567890abcdef")
    parser.add_argument("-i", "--id", help="ID do projeto com jobs a executar", metavar="123")

    args = parser.parse_args()

    executor = GitlabJob(token=args.get("private_token"), 
                         project_id=args.get("id"),         
                         base_url=args.get("url_base"))
    status_list = executor.play_all()

    for job in status_list.items():
        print(f"Status de inicialização do job {job[0]} : {job[1]}")
