// Numeric behavior only: native unit/object valuations and reserve budget.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
import java.math.BigInteger;
public class ZNativeAiScoringOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,W=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 byte[] weights;
 byte[] robots;
 void weights(EmulatorHelper e)throws Exception{e.writeMemory(toAddr(0x4a8980),weights);}
 int sample(int profile,int variant,int field){return (profile*13+variant*31+field*17)&255;}
 public void run()throws Exception{weights=Files.readAllBytes(Paths.get("C:/GOG Games/Z/mult.dat"));robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[] traits={0,2,4,7,0x3a,0x3e,0x7a};int[] weapon={5,6,7,0x20,0x21,0x23,0x24,0x25,0x26};
  for(int profile=0;profile<19;profile++)for(int cls:new int[]{0,1,2,3,15,16})for(int variant=0;variant<6;variant++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    weights(e);e.writeMemory(toAddr(U),new byte[0x2000]);put(e,U+8,4,W);put(e,U+0xa1,1,profile);put(e,U+0x7c,1,cls);
    for(int i=0;i<traits.length;i++)put(e,U+traits[i],1,sample(profile,variant,i));
    put(e,U+0x99,1,variant*43);put(e,U+0x65,1,sample(profile,variant,7));put(e,U+0x78,1,sample(profile,variant,8));put(e,U+0x79,1,sample(profile,variant,9));put(e,U+0x74,1,sample(profile,variant,10));
    for(int i=0;i<weapon.length;i++)put(e,W+weapon[i],1,sample(profile,variant,11+i));put(e,W+8,2,300-variant*117);
    invoke(e,0x41a560);out.println("{\"kind\":\"unit\",\"profile\":"+profile+",\"class\":"+cls+",\"variant\":"+variant+",\"score\":"+e.readRegister("EAX").intValue()+"}");
   }finally{e.dispose();}
  }
  for(int raw=0;raw<19;raw++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    weights(e);e.writeMemory(toAddr(U),new byte[0x2000]);put(e,U+8,4,W);int r=raw*44,q=0x55f0+(robots[r+34]&255)*42,profile=robots[q+1]&255;
    int cls=profile==17||profile==18?0:profile==1||profile>=7&&profile<=14?1:profile==2||profile>=4&&profile<=6?2:profile;
    put(e,U+0xa1,1,profile);put(e,U+0x7c,1,cls);int[] offsets={28,29,30,31,32,29,36};for(int i=0;i<traits.length;i++)put(e,U+traits[i],1,robots[r+offsets[i]]&255);
    put(e,U+0x65,1,robots[q+32]&255);put(e,U+0x78,1,robots[q+34]&255);put(e,U+0x79,1,robots[q+35]&255);int range=((robots[q+2]&255)|((robots[q+3]&255)<<8))>>3;put(e,U+0x74,1,range);
    put(e,W+5,1,range);put(e,W+6,1,robots[q+32]&255);put(e,W+7,1,robots[q+4]&255);put(e,W+8,2,robots[q+5]*25);put(e,W+0x20,1,robots[q+34]&255);put(e,W+0x21,1,robots[q+35]&255);put(e,W+0x23,1,robots[q+37]&255);put(e,W+0x24,1,robots[q+38]&255);put(e,W+0x25,1,robots[q+39]&255);put(e,W+0x26,1,robots[q+36]&255);
    invoke(e,0x41a560);out.println("{\"kind\":\"default\",\"raw\":"+raw+",\"profile\":"+profile+",\"score\":"+e.readRegister("EAX").intValue()+"}");
   }finally{e.dispose();}
  }
  for(int typ:new int[]{5,8,21,22,23,24,26,29,31,33,35,37,0})for(int variant=0;variant<6;variant++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    weights(e);e.writeMemory(toAddr(U),new byte[0xc0]);int[] fields={0xe,0x10,0xf,0x11,0x18,0x19,0x1e,0x1b,0x1c,0x1d};for(int i=0;i<fields.length;i++)put(e,U+fields[i],1,sample(typ,variant,i));
    e.writeRegister("EDX",typ);invoke(e,0x41aae0);out.println("{\"kind\":\"object\",\"type\":"+typ+",\"variant\":"+variant+",\"score\":"+e.readRegister("EAX").intValue()+"}");
   }finally{e.dispose();}
  }
  for(int dx:new int[]{-65,-17,-1,0,1,17,65})for(int dy:new int[]{-65,-17,-1,0,1,17,65})for(int score:new int[]{-131071,0,1,3571,131071,2147483647}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[30]);e.writeMemory(toAddr(0x5c3f0c),new byte[30*64]);
    put(e,0x5c3f0c,2,1);put(e,0x5c3f0c+0x12,1,40);put(e,0x5c3f0c+0x13,1,40);put(e,0x5c3f0c+0x18,1,1);
    put(e,U+0x12,1,40+dx);put(e,U+0x13,1,40+dy);put(e,0x4a768c,2,1);put(e,0x4a7790,4,U);put(e,0x4a7794,4,score);put(e,0x4a69eb,1,0);e.setBreakpoint(toAddr(0x473264));
    invoke(e,0x413ca0);while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x473264)throw new Exception("Unexpected priority stop");double v=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(v))));resume(e);}
    out.println("{\"kind\":\"priority\",\"dx\":"+dx+",\"dy\":"+dy+",\"score\":"+score+",\"result\":"+(int)read(e,0x4a7794,4)+"}");
   }finally{e.dispose();}
  }
  for(int priority:new int[]{-131071,0,1,499,500,3571,131071,2147483647})for(int reserve:new int[]{0,1,3571,100000})for(int attack:new int[]{0,1,2500,200000}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(0x5c3f0c),new byte[30*64]);put(e,0x5c3f0c+8,1,1);put(e,0x5c3f0c+0x19,4,priority);put(e,0x5c3f00,4,Float.floatToRawIntBits(reserve));put(e,0x5c3ef8,4,Float.floatToRawIntBits(attack));e.setBreakpoint(toAddr(0x414610));e.setBreakpoint(toAddr(0x4159d2));invoke(e,0x415950);
    while(e.readRegister("EIP").longValue()!=0x4159d2){if(e.readRegister("EIP").longValue()!=0x414610)throw new Exception("Unexpected defense stop");e.writeRegister("EAX",0);resume(e);}
    out.println("{\"kind\":\"defense\",\"priority\":"+priority+",\"reserve\":"+reserve+",\"attack\":"+attack+",\"result\":"+e.readRegister("EBX").intValue()+"}");
   }finally{e.dispose();}
  }
  for(int reserve:new int[]{-128,-1,0,25,50,100,127})for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(0x5c3f0c),new byte[30*64]);for(int i=0;i<64;i++)put(e,0x5c3f0c+i*30,2,65535);
    if((scenario&1)!=0){put(e,0x5c3f0c+63*30,2,1);put(e,0x5c3f0c+63*30+24,1,1);}
    for(int i=0;i<6;i++){long p=0x5c3f0c+i*30;put(e,p,2,i<scenario%7?1:65535);put(e,p+24,1,((scenario>>(i%5))&1)!=0?128:0);put(e,p+10,2,scenario*3571+i*5303);put(e,p+12,2,scenario*7919+i*157);}
    put(e,0x5c468d,1,reserve);invoke(e,0x415d60);
    out.println("{\"kind\":\"pressure\",\"reserve\":"+reserve+",\"scenario\":"+scenario+",\"surplusBits\":"+read(e,0x5c3f08,4)+",\"deficitBits\":"+read(e,0x4a811c,4)+"}");
   }finally{e.dispose();}
  }
  // The numerical base and update gate execute natively; separately emulated
  // object/building valuations are stubbed out for this composition trace.
  for(int scenario=0;scenario<128;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(0x5c3f0c),new byte[30*64]);for(int i=0;i<64;i++)put(e,0x5c3f0c+i*30,2,65535);long p=0x5c3f0c;
    int lower=scenario%3-1,population=(byte)(scenario*43),weight=(byte)(scenario*17),human=(short)(scenario*3571),before=scenario*941;
    put(e,p,2,lower);put(e,p+9,1,population);put(e,p+10,2,human);put(e,p+25,4,before);put(e,p+29,1,(scenario>>3)&1);put(e,0x5c47ab,1,weight);put(e,0x4a69ef,1,(scenario>>4)&1);
    e.setBreakpoint(toAddr(0x41a150));e.setBreakpoint(toAddr(0x41a230));invoke(e,0x41a290);
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a!=0x41a150&&a!=0x41a230)throw new Exception("Unexpected composition stop");resume(e);}
    out.println("{\"kind\":\"composition\",\"scenario\":"+scenario+",\"result\":"+(int)read(e,p+25,4)+"}");
   }finally{e.dispose();}
  }
  // Stop after the budget arithmetic, before the independent reserve flag loop.
  for(int aggression:new int[]{0,25,50,100,127,255})for(int own:new int[]{0,1,3571,131071})for(int surplus:new int[]{0,1000,100000})for(int deficit:new int[]{0,2500,200000}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(0x5c47b8),new byte[0x18*240]);e.writeMemory(toAddr(0x60ec00),new byte[0x48*240]);
    for(int i=0;i<240;i++)put(e,0x60ec13+i*0x48,1,255);
    put(e,0x5c47cd,1,1);put(e,0x5c468d,1,aggression);put(e,0x5c3f08,4,Float.floatToRawIntBits(surplus));put(e,0x4a811c,4,Float.floatToRawIntBits(deficit));
    e.setBreakpoint(toAddr(0x41a800));e.setBreakpoint(toAddr(0x41a8b0));e.setBreakpoint(toAddr(0x416fb9));
    invoke(e,0x416e10);
    while(e.readRegister("EIP").longValue()!=RET&&e.readRegister("EIP").longValue()!=0x416fb9){long a=e.readRegister("EIP").longValue();if(a==0x41a800)e.writeRegister("EAX",own);else if(a!=0x41a8b0)throw new Exception("Unexpected budget stop "+Long.toHexString(a));resume(e);}
    out.println("{\"kind\":\"budget\",\"aggression\":"+aggression+",\"own\":"+own+",\"surplus\":"+surplus+",\"deficit\":"+deficit+",\"reserveBits\":"+read(e,0x5c3f00,4)+",\"attackBits\":"+read(e,0x5c3ef8,4)+"}");
   }finally{e.dispose();}
  }
 }}
}
