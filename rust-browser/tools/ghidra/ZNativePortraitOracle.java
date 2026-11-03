// Read-only68c80/68d30 selection/coordinate oracle.65620's GPU boundary is
//intercepted only to record its sprite and integer draw coordinates.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.util.*;
public class ZNativePortraitOracle extends GhidraScript{
 static final long STACK=0x7fff0000L,RET=0x12345000L,U=0x1000000L;
 void put(EmulatorHelper e,long a,int n,long v){e.writeMemoryValue(toAddr(a),n,v);}
 long word(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void begin(EmulatorHelper e,long entry,long cx,long dx)throws Exception{e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("ESP",STACK);put(e,STACK,4,RET);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x465620));int[]bases={137,177,217,257,297};long[]globals={0x64d7ec,0x64d7f8,0x64d7f4,0x64d7f0,0x64d7fc};for(int i=0;i<5;i++)put(e,globals[i],2,bases[i]);put(e,0x64d810,4,131);put(e,0x622c7a,2,0);put(e,0x622dc4,1,0);put(e,0x632158,4,U);
  for(int head=0;head<5;head++)for(int world=0;world<5;world++)for(int vehicle=0;vehicle<2;vehicle++)for(int seed=0;seed<256;seed++){
   int packed=(seed*40503+17)&65535;begin(e,0x468c80,packed,bases[head]);ArrayList<Long> row=new ArrayList<>();for(long n:new long[]{head,world,vehicle,seed,packed})row.add(n);for(int i=0;i<10;i++)row.add((long)(short)word(e,0x64f7c0+i*2,2));
   put(e,0x625eae,1,world);put(e,U+0x7c,1,vehicle==1?1:0);put(e,STACK+4,4,0);put(e,STACK+8,4,vehicle);begin(e,0x468d30,12,20);ArrayList<Long> draws=new ArrayList<>();int count=0;
   while(e.readRegister("EIP").longValue()==0x465620){long sp=e.readRegister("ESP").longValue();draws.add(word(e,sp+4,4)&65535);draws.add((long)(short)e.readRegister("ECX").longValue());draws.add((long)(short)e.readRegister("EDX").longValue());long resume=word(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(resume),null,monitor))throw new Exception(e.getLastError());if(++count>8)throw new Exception("Unexpected portrait draw loop");}
   row.add((long)count);row.addAll(draws);for(int i=0;i<row.size();i++){if(i!=0)out.print(',');out.print(row.get(i));}out.println();
  }
 }finally{e.dispose();}}}
}
