cmd_/home/maoyutofu/work/cicv-r4l-3-maoyutofu/r4l_experiment/driver/002_completion/Module.symvers :=  sed 's/ko$$/o/'  /home/maoyutofu/work/cicv-r4l-3-maoyutofu/r4l_experiment/driver/002_completion/modules.order | scripts/mod/modpost      -o /home/maoyutofu/work/cicv-r4l-3-maoyutofu/r4l_experiment/driver/002_completion/Module.symvers -e -i Module.symvers -T - 
