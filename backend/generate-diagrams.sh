#!/bin/bash

# 果蔬检测系统 UML 图表生成脚本
# 使用 PlantUML 生成项目的所有架构图表
# 
# 使用方法:
#   ./generate-diagrams.sh           # 生成所有图表
#   ./generate-diagrams.sh sequence  # 只生成序列图
#   ./generate-diagrams.sh all       # 生成所有图表（默认）

set -e

# 检查是否安装了Java和PlantUML
check_dependencies() {
    echo "检查依赖..."
    
    if ! command -v java &> /dev/null; then
        echo "错误: 未找到Java运行环境，请安装Java 11或更高版本"
        exit 1
    fi
    
    if [ ! -f "plantuml.jar" ]; then
        echo "PlantUML JAR文件未找到，正在下载..."
        wget -O plantuml.jar https://github.com/plantuml/plantuml/releases/download/v1.2024.7/plantuml-1.2024.7.jar
    fi
    
    echo "依赖检查完成"
}

# 生成指定的UML图表
generate_diagram() {
    local diagram_file=$1
    local diagram_name=$2
    
    if [ -f "$diagram_file" ]; then
        echo "生成 $diagram_name..."
        java -jar plantuml.jar "$diagram_file"
        if [ $? -eq 0 ]; then
            echo "✓ $diagram_name 生成成功"
        else
            echo "✗ $diagram_name 生成失败"
            return 1
        fi
    else
        echo "警告: 文件 $diagram_file 不存在，跳过 $diagram_name"
    fi
}

# 移动生成的文件到输出目录
organize_files() {
    echo "整理文件..."
    
    # 创建输出目录（如果不存在）
    mkdir -p uml-output
    
    # 移动PNG文件
    if ls *.png &> /dev/null; then
        mv *.png uml-output/
        echo "PNG文件已移动到 uml-output/ 目录"
    fi
    
    # 移动PUML源文件
    if ls *.puml &> /dev/null; then
        cp *.puml uml-output/
        echo "PUML源文件已复制到 uml-output/ 目录"
    fi
}

# 显示生成的文件列表
show_results() {
    echo ""
    echo "生成的图表文件:"
    echo "================"
    ls -la uml-output/*.png 2>/dev/null | while read -r line; do
        filename=$(echo "$line" | awk '{print $9}')
        size=$(echo "$line" | awk '{print $5}')
        echo "  $(basename "$filename") (${size} bytes)"
    done
    echo ""
    echo "源文件:"
    echo "======="
    ls -la uml-output/*.puml 2>/dev/null | while read -r line; do
        filename=$(echo "$line" | awk '{print $9}')
        echo "  $(basename "$filename")"
    done
}

# 主函数
main() {
    local target=${1:-all}
    
    echo "果蔬检测系统 UML 图表生成工具"
    echo "================================="
    echo ""
    
    check_dependencies
    
    case $target in
        "sequence")
            generate_diagram "sequence-diagram.puml" "序列图"
            ;;
        "architecture")
            generate_diagram "architecture-diagram.puml" "架构图"
            ;;
        "workflow")
            generate_diagram "workflow-diagram.puml" "工作流图"
            ;;
        "dataflow")
            generate_diagram "dataflow-diagram.puml" "数据流图"
            ;;
        "all"|*)
            echo "生成所有图表..."
            generate_diagram "sequence-diagram.puml" "序列图"
            generate_diagram "architecture-diagram.puml" "架构图"  
            generate_diagram "workflow-diagram.puml" "工作流图"
            generate_diagram "dataflow-diagram.puml" "数据流图"
            ;;
    esac
    
    organize_files
    show_results
    
    echo ""
    echo "✅ 图表生成完成！"
    echo "📁 所有文件保存在 uml-output/ 目录中"
    echo ""
    echo "使用方法:"
    echo "  - 在文档中插入图片: ![图片说明](uml-output/图片文件名.png)"
    echo "  - 修改图表: 编辑 .puml 文件后重新运行此脚本"
    echo "  - 查看原始PlantUML代码: 查看 uml-output/ 目录中的 .puml 文件"
}

# 执行主函数
main "$@"