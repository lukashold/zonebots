const path=require('node:path'),{spawnSync}=require('node:child_process');
for(const file of ['check-exe-reader.py','check-cpu-import.py','check-native-movement-source.py']){
 const result=spawnSync('python',[path.join(__dirname,file)],{encoding:'utf8',windowsHide:true});
 process.stdout.write(result.stdout||'');process.stderr.write(result.stderr||'');if(result.status!==0)process.exit(result.status||1);
}
