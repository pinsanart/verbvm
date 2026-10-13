import {House, BookOpen, Layers, Settings2 } from 'lucide-react'
import { NavigationButtonType } from '../types/NavigationButton'

export const buttons: NavigationButtonType[] = [
    {icon: House, labelKey: 'nav.home', to: '/'},
    {icon: BookOpen, labelKey: 'nav.review', to: '/review'},
    {icon: Layers, labelKey: 'nav.decks', to: '/decks'},
    {icon: Settings2, labelKey: 'nav.settings', to: '/settings'},
]
