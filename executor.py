from jobmail import Jobmail
from gitlabjob import GitlabJob
from os import environ
from time import sleep

if __name__ == "__main__":

    gitlabjob = GitlabJob()

    if environ.get("GROUP_ID"):
        lst_projs = gitlabjob.group_projs(environ.get("GROUP_ID"))
    else:
        lst_projs = [environ.get("PROJECT_ID")]

    for proj in lst_projs:
        print("="*5, f" PROJETO ID: {proj}")
        if not proj: continue
        gitlabjob.project_id = proj

        jobmail = Jobmail()

        ### Limpa fila de pipelines, permanecendo apenas o último
        send_email = False
        n_control = 0
        list_pipelines = gitlabjob.get_pipelines()
        while len(list_pipelines) > 1:
            n_control += 1
            if n_control > 5: break
            for pipe in list_pipelines[:-1]:
                gitlabjob.delete_pipeline(pipe)
                send_email = True
                jobmail.msg_append(f"Excluída pipeline duplicada: {pipe}")
            sleep(5)
            list_pipelines = gitlabjob.get_pipelines()

        projname = gitlabjob.get_jobinfo().get("nome_projeto")
        if send_email:
            jobmail.remetente(environ.get("SMTP_FROM", "gitlabjob@mail.com"))
            jobmail.destino_add(environ["SMTP_TO"])
            jobmail.assunto(f"[GitlabJob] Exclusão de pipeline ({projname})")
            jobmail.send()

        jobs = set(gitlabjob.get_jobs(filtro={"status": "manual"}))

        for job in jobs:
            print(f"Executando job {job}")
            jobmail = Jobmail()
            jobmail.remetente(environ.get("SMTP_FROM", "gitlabjob@mail.com"))
            jobmail.assunto(f"[GitlabJob] Status do job {job} ({projname})")

            ### Coleta informações da tarefa
            job_infos = gitlabjob.get_jobinfo(job)
            for dado in job_infos.items():
                jobmail.msg_append(f"{dado[0]}: {dado[1]}")

            jobmail.destino_add(environ.get("SMTP_TO"))
            jobmail.destino_add(job_infos.get("user_mail"))

            ### Condição para execução
            executavel = False
            tag_wanted = job_infos.get("versao_tag")
            projorig_id = job_infos.get("source_id")

            if projorig_id:
                proj_tags = gitlabjob.get_tags(projorig_id)
                if tag_wanted in proj_tags:
                    executavel = True
                else:
                    jobmail.msg_append(
                    f"tag {tag_wanted} não encontrada no projeto de id {projorig_id}")
                    executavel = False
            else:
                jobmail.msg_append("ID do projeto origem não obtida")
                executavel = False

            ### Executa tarefa
            if executavel:
                start_job = gitlabjob.play_job(job)
                sleep(2)
                jobmail.msg_append(
                    f"Código de resposta HTTP para execução: {start_job}")
            else:
                jobmail.msg_append(f"Tarefa {job} não executada.")

            jobmail.send()
