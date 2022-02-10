from gitlabjob import GitlabJob
from os import environ
from utils import load_conf

if __name__ == "__main__":
    load_conf(".env")

    gitlabjob = GitlabJob()

    if environ.get("GROUP_ID"):
        lst_projs = gitlabjob.group_projs(environ.get("GROUP_ID"))
    else:
        lst_projs = [environ.get("PROJECT_ID")]

    for proj in lst_projs:

        print("="*5, f" PROJETO ID: {proj}")
        gitlabjob.project_id = proj

        jobs = sorted(set(gitlabjob.get_jobs(filtro={"status": "manual"})))

        for job in jobs:
            print(f"Executando job {job}")

            job_infos = gitlabjob.get_jobinfo(job)
            print(job_infos)

            start_job = gitlabjob.play_job(job)