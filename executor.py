from jobmail import Jobmail
from gitlabjob import GitlabJob
from os import environ

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

        status_dict = gitlabjob.play_all(filtro={"status":"manual"})

        for job in status_dict.items():
            print(f"Executando job {job[0]}")
            jobmail = Jobmail()
            jobmail.remetente(environ.get("SMTP_FROM", "gitlabjob@mail.com"))

            job_info = gitlabjob.get_jobinfo(job[0])
            job_info["inicio"] = "OK" if job[1] == 200 else job[1]

            for dado in job_info.items():
                jobmail.msg_append(f"{dado[0]}: {dado[1]}")

            try:
                jobmail.destino(",".join([environ.get("SMTP_TO"), 
                                  job_info.get("user_mail")]))
            except TypeError:
                jobmail.destino(environ.get("SMTP_TO"))

            destinatarios = jobmail.msg.get("To")
            print(f"Enviando email para {destinatarios}")
            jobmail.assunto(f"[GitlabJob] Status do job {job[0]}")

            jobmail.send()
