.\nntask1.exe --input1 tests\t1_input.txt --output1 t1_output.xml

.\nntask2.exe --input1 tests\t2_input.xml --output1 t2_output1.xml
.\nntask2.exe --input1 tests\t2_input_cycle.xml --output1 t2_output2.xml

.\nntask3.exe --input1 tests\t1_output.xml --input2 tests\t3_ops.json --output1 t3_output.txt

.\nntask4.exe convert --weights tests\t4_w.json --output tests\t4_model.json
.\nntask4.exe run --model tests\t4_model.json --input tests\t4_x.txt --output tests\t4_output.txt
