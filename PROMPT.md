GH_RANGE=1-13
BRANCH=main

1. read ./README.md
2. look at gh commits to see which gh issues in <GH_RANGE> have already been completed
3. find the NEXT_GH_ISSUE sequentially
4. read <NEXT_GH_ISSUE> detail
5. impl fix, commit message in a way that will close gh issue when pushed to main
6. push to <BRANCH>

iteration is finished when you've pushed to <BRANCH>
loop is finished when all the issues in <GH_RANGE> are finished


IMPORTANT:

- One gh issue per iteration
- Do not commit any ralph files
- Use `ralph` CLI to setup the next task before finishing your iteration
