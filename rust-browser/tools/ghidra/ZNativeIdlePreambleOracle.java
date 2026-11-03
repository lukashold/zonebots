// Isolated native44d920 preamble gates. World handlers are boundaries;
// the parameterized RNG return checks inclusive/strict gate comparisons.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeIdlePreambleOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,G=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int role:new int[]{-1,0,1})for(int flags:new int[]{0,16,32,48})for(int attached:new int[]{0,8})for(int target:new int[]{255,7})for(int suppressed=0;suppressed<2;suppressed++)for(int hold=0;hold<2;hold++)for(int force=0;force<2;force++)for(int aim:new int[]{0,60,120})for(int roll:new int[]{0,60,127}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,0x632194,4,G);put(e,G+0x3b,1,target);put(e,G+0x3d,1,hold);put(e,G+0x3e,1,suppressed);put(e,G+0x3f,1,force);put(e,U+0x5e,1,role);put(e,U+0x64,1,flags);put(e,U+0x31,1,attached);put(e,U+2,1,aim);put(e,U+0x50,2,7);put(e,0x5e30c8+7*0xc0+0x2c,2,500);put(e,0x5e30c8+7*0xc0+0x2e,2,600);
    for(long a:new long[]{RET,0x44da46,0x458620,0x460910,0x44a540,0x44ca50,0x459c60,0x44d570})e.setBreakpoint(toAddr(a));
    e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);if(!e.run(toAddr(0x44d920),null,monitor))throw new Exception(e.getLastError());int calls=0,refresh=0,gate=0,wx=0,wy=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x458620){refresh++;resume(e);continue;}if(a==0x460910){calls++;e.writeRegister("EAX",roll);resume(e);continue;}
     if(a==0x44da46)gate=1;else if(a==0x44a540)gate=2;else if(a==0x44ca50)gate=3;else if(a==0x459c60){gate=4;wx=(short)e.readRegister("EDX").intValue();wy=(short)read(e,e.readRegister("ESP").longValue()+4,4);}else if(a==0x44d570)gate=5;else throw new Exception("Unexpected preamble stop");break;
    }
    out.println("{\"role\":"+role+",\"flags\":"+flags+",\"attached\":"+attached+",\"target\":"+target+",\"suppressed\":"+suppressed+",\"hold\":"+hold+",\"force\":"+force+",\"aim\":"+aim+",\"roll\":"+roll+",\"gate\":"+gate+",\"calls\":"+calls+",\"refresh\":"+refresh+",\"wx\":"+wx+",\"wy\":"+wy+"}");
   }finally{e.dispose();}
  }
 }}
}
