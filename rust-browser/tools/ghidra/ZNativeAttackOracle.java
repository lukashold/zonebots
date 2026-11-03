// Read-only numeric behavior oracle for the locally owned GOG executable.
// Run against an existing Ghidra12.2 project with -readOnly -noanalysis.
// Emulator writes stay in emulated memory; no program or resources are exported.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAttackOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,U=0x1000000L;
 long word(EmulatorHelper e,long a){byte[]b=e.readMemory(toAddr(a),4);return(b[0]&255L)|((b[1]&255L)<<8)|((b[2]&255L)<<16)|((b[3]&255L)<<24);}
 void invoke(EmulatorHelper e,long entry)throws Exception{
  e.writeRegister("ESP",STACK);e.writeMemoryValue(toAddr(STACK),4,RET);e.writeMemoryValue(toAddr(STACK+4),4,U);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
 }
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
   for(long[]test:new long[][]{{0x449e70,0x456760,3},{0x448aa0,0x456760,1},{0x448950,0x456990,1},{0x448e80,0x456840,1},{0x457400,0x457300,20},{0x457270,0x457120,5}}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[192]);e.writeMemoryValue(toAddr(U+0x3c),1,test[2]);e.writeMemoryValue(toAddr(U+0x78),1,80);
     e.setBreakpoint(toAddr(test[1]));e.setBreakpoint(toAddr(0x41f4d0));
     for(int t=1;t<=100;t++){
      invoke(e,test[0]);
      if(e.readRegister("EIP").longValue()==0x41f4d0){long esp=e.readRegister("ESP").longValue();long resume=word(e,esp);e.writeRegister("ESP",esp+4);e.writeRegister("EAX",1);if(!e.run(toAddr(resume),null,monitor))throw new Exception(e.getLastError());}
      if(e.readRegister("EIP").longValue()==test[1]){out.println("{\"kind\":\"attack\",\"handler\":"+test[0]+",\"release\":"+t+"}");break;}
     }
    }finally{e.dispose();}
   }
   for(int control:new int[]{0,55,100,127,140})for(int score:new int[]{-128,0,1000}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeRegister("ECX",371);invoke(e,0x460850);long[]before=new long[5];for(int i=0;i<5;i++)before[i]=word(e,0x622cdcL+i*4);
     e.writeMemoryValue(toAddr(U),4,score);e.writeMemoryValue(toAddr(0x4a897c),2,control);e.writeRegister("ECX",U);invoke(e,0x41a8b0);
     boolean saved=true;for(int i=0;i<5;i++)saved&=before[i]==word(e,0x622cdcL+i*4);
     out.println("{\"kind\":\"score\",\"control\":"+control+",\"input\":"+score+",\"result\":"+(int)word(e,U)+",\"rngRestored\":"+saved+"}");
    }finally{e.dispose();}
   }
  }
 }
}